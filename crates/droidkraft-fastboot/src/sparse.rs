//! Android sparse-image splitting (streamed from disk).
//!
//! Raw or sparse images larger than the device `max-download-size` are
//! re-encoded as several sparse "pieces", each ≤ the limit and each spanning
//! the whole output (leading/trailing DONT_CARE chunks), exactly like
//! libsparse's `resparse` does for `fastboot flash`.

use std::io::{self, Read, Seek, SeekFrom};

const MAGIC: u32 = 0xED26_FF3A;
const FILE_HDR: u64 = 28;
const CHUNK_HDR: u64 = 12;
const BLK: u64 = 4096;
const RAW: u16 = 0xCAC1;
const FILL: u16 = 0xCAC2;
const SKIP: u16 = 0xCAC3;
const CRC: u16 = 0xCAC4;

/// A byte run inside a piece.
#[derive(Debug, Clone)]
pub enum Seg {
    Bytes(Vec<u8>),
    File { off: u64, len: u64 },
    Zeros(u64),
}

/// One download unit.
#[derive(Debug, Clone)]
pub struct Piece {
    pub segs: Vec<Seg>,
}

impl Piece {
    pub fn len(&self) -> u64 {
        self.segs
            .iter()
            .map(|s| match s {
                Seg::Bytes(b) => b.len() as u64,
                Seg::File { len, .. } | Seg::Zeros(len) => *len,
            })
            .sum()
    }
}

#[derive(Debug, Clone, Copy)]
enum Chunk {
    Skip(u64),
    Fill(u64, u32),
    Raw {
        off: u64,
        data_len: u64,
        blocks: u64,
    },
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn is_sparse<R: Read + Seek>(r: &mut R) -> io::Result<bool> {
    r.seek(SeekFrom::Start(0))?;
    let mut m = [0u8; 4];
    match r.read_exact(&mut m) {
        Ok(()) => Ok(u32::from_le_bytes(m) == MAGIC),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(e) => Err(e),
    }
}

/// Parse a sparse file into (block size, total blocks, chunks).
fn parse_sparse<R: Read + Seek>(r: &mut R) -> io::Result<(u64, u64, Vec<Chunk>)> {
    r.seek(SeekFrom::Start(0))?;
    let mut h = [0u8; FILE_HDR as usize];
    r.read_exact(&mut h)?;
    if u32le(&h, 0) != MAGIC || u16le(&h, 4) != 1 {
        return Err(bad("unsupported sparse header"));
    }
    let (fhs, chs) = (u16le(&h, 8) as u64, u16le(&h, 10) as u64);
    let (blk, total, nchunks) = (u32le(&h, 12) as u64, u32le(&h, 16) as u64, u32le(&h, 20));
    if blk == 0 || fhs < FILE_HDR || chs < CHUNK_HDR {
        return Err(bad("bad sparse header"));
    }
    let mut pos = fhs;
    let mut out = Vec::new();
    for _ in 0..nchunks {
        r.seek(SeekFrom::Start(pos))?;
        let mut c = [0u8; CHUNK_HDR as usize];
        r.read_exact(&mut c)?;
        let (ty, blocks, tsz) = (u16le(&c, 0), u32le(&c, 4) as u64, u32le(&c, 8) as u64);
        let data = pos + chs;
        match ty {
            RAW => out.push(Chunk::Raw {
                off: data,
                data_len: blocks * blk,
                blocks,
            }),
            FILL => {
                r.seek(SeekFrom::Start(data))?;
                let mut v = [0u8; 4];
                r.read_exact(&mut v)?;
                out.push(Chunk::Fill(blocks, u32::from_le_bytes(v)));
            }
            SKIP => out.push(Chunk::Skip(blocks)),
            CRC => {}
            _ => return Err(bad("unknown sparse chunk type")),
        }
        if tsz < chs {
            return Err(bad("bad chunk size"));
        }
        pos += tsz;
    }
    Ok((blk, total, out))
}

/// Size of the image once expanded onto the partition.
pub fn expanded_size<R: Read + Seek>(r: &mut R) -> io::Result<u64> {
    if is_sparse(r)? {
        let (blk, total, _) = parse_sparse(r)?;
        Ok(blk * total)
    } else {
        r.seek(SeekFrom::End(0))
    }
}

/// Plan the downloads for an image. Images that already fit are sent as-is.
pub fn plan<R: Read + Seek>(r: &mut R, max: Option<u64>) -> io::Result<Vec<Piece>> {
    let len = r.seek(SeekFrom::End(0))?;
    if max.is_none_or(|m| len <= m) {
        return Ok(vec![Piece {
            segs: vec![Seg::File { off: 0, len }],
        }]);
    }
    let max = max.unwrap_or(u64::MAX);
    let (blk, total, chunks) = if is_sparse(r)? {
        parse_sparse(r)?
    } else {
        let blocks = len.div_ceil(BLK);
        (
            BLK,
            blocks,
            vec![Chunk::Raw {
                off: 0,
                data_len: len,
                blocks,
            }],
        )
    };
    pack(&chunks, blk, total, max)
}

fn encode(chunks: &[Chunk], blk: u64, total: u64) -> Piece {
    let mut segs = Vec::new();
    let mut h = Vec::with_capacity(FILE_HDR as usize);
    h.extend(MAGIC.to_le_bytes());
    h.extend(1u16.to_le_bytes());
    h.extend(0u16.to_le_bytes());
    h.extend((FILE_HDR as u16).to_le_bytes());
    h.extend((CHUNK_HDR as u16).to_le_bytes());
    h.extend((blk as u32).to_le_bytes());
    h.extend((total as u32).to_le_bytes());
    h.extend((chunks.len() as u32).to_le_bytes());
    h.extend(0u32.to_le_bytes());
    segs.push(Seg::Bytes(h));
    let hdr = |ty: u16, blocks: u64, tsz: u64| {
        let mut c = Vec::with_capacity(CHUNK_HDR as usize);
        c.extend(ty.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend((blocks as u32).to_le_bytes());
        c.extend((tsz as u32).to_le_bytes());
        Seg::Bytes(c)
    };
    for c in chunks {
        match *c {
            Chunk::Skip(n) => segs.push(hdr(SKIP, n, CHUNK_HDR)),
            Chunk::Fill(n, v) => {
                segs.push(hdr(FILL, n, CHUNK_HDR + 4));
                segs.push(Seg::Bytes(v.to_le_bytes().to_vec()));
            }
            Chunk::Raw {
                off,
                data_len,
                blocks,
            } => {
                segs.push(hdr(RAW, blocks, CHUNK_HDR + blocks * blk));
                segs.push(Seg::File { off, len: data_len });
                if blocks * blk > data_len {
                    segs.push(Seg::Zeros(blocks * blk - data_len));
                }
            }
        }
    }
    Piece { segs }
}

fn pack(items: &[Chunk], blk: u64, total: u64, limit: u64) -> io::Result<Vec<Piece>> {
    // Header + reserve for a trailing DONT_CARE chunk.
    let base = FILE_HDR + CHUNK_HDR;
    let mut pieces = Vec::new();
    let (mut cur, mut ppos, mut size) = (0u64, 0u64, base);
    let mut piece: Vec<Chunk> = Vec::new();

    let flush =
        |piece: &mut Vec<Chunk>, ppos: &mut u64, size: &mut u64, pieces: &mut Vec<Piece>| {
            if piece.is_empty() {
                return;
            }
            let mut chunks = std::mem::take(piece);
            if *ppos < total {
                chunks.push(Chunk::Skip(total - *ppos));
            }
            pieces.push(encode(&chunks, blk, total));
            *ppos = 0;
            *size = base;
        };

    for item in items {
        match *item {
            Chunk::Skip(n) => cur += n,
            Chunk::Fill(n, v) => {
                let need = |ppos: u64| if cur > ppos { CHUNK_HDR } else { 0 } + CHUNK_HDR + 4;
                if size + need(ppos) > limit {
                    flush(&mut piece, &mut ppos, &mut size, &mut pieces);
                }
                if cur > ppos {
                    piece.push(Chunk::Skip(cur - ppos));
                }
                size += need(ppos);
                piece.push(Chunk::Fill(n, v));
                cur += n;
                ppos = cur;
            }
            Chunk::Raw {
                off,
                data_len,
                blocks,
            } => {
                let (mut off, mut data_left, mut left) = (off, data_len, blocks);
                while left > 0 {
                    let gap = if cur > ppos { CHUNK_HDR } else { 0 };
                    let avail = limit.saturating_sub(size + gap + CHUNK_HDR);
                    let nb = left.min(avail / blk);
                    if nb == 0 {
                        if piece.is_empty() {
                            return Err(bad("max-download-size too small for sparse split"));
                        }
                        flush(&mut piece, &mut ppos, &mut size, &mut pieces);
                        continue;
                    }
                    if gap > 0 {
                        piece.push(Chunk::Skip(cur - ppos));
                    }
                    let dl = data_left.min(nb * blk);
                    piece.push(Chunk::Raw {
                        off,
                        data_len: dl,
                        blocks: nb,
                    });
                    size += gap + CHUNK_HDR + nb * blk;
                    off += dl;
                    data_left -= dl;
                    left -= nb;
                    cur += nb;
                    ppos = cur;
                }
            }
        }
    }
    flush(&mut piece, &mut ppos, &mut size, &mut pieces);
    if pieces.is_empty() {
        pieces.push(encode(&[Chunk::Skip(total)], blk, total));
    }
    Ok(pieces)
}

/// Stream a piece to `sink` in buffers of exactly `chunk` bytes (last one shorter).
pub fn stream<R: Read + Seek>(
    piece: &Piece,
    r: &mut R,
    chunk: usize,
    sink: &mut dyn FnMut(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    let chunk = chunk.max(1);
    let mut buf: Vec<u8> = Vec::with_capacity(chunk);
    let mut feed = |mut data: &[u8], buf: &mut Vec<u8>| -> Result<(), String> {
        while !data.is_empty() {
            let n = (chunk - buf.len()).min(data.len());
            buf.extend_from_slice(&data[..n]);
            data = &data[n..];
            if buf.len() == chunk {
                sink(buf)?;
                buf.clear();
            }
        }
        Ok(())
    };
    let mut tmp = vec![0u8; 256 * 1024];
    for seg in &piece.segs {
        match seg {
            Seg::Bytes(b) => feed(b, &mut buf)?,
            Seg::Zeros(n) => {
                let mut left = *n;
                tmp.fill(0);
                while left > 0 {
                    let k = left.min(tmp.len() as u64) as usize;
                    feed(&tmp[..k], &mut buf)?;
                    left -= k as u64;
                }
            }
            Seg::File { off, len } => {
                r.seek(SeekFrom::Start(*off)).map_err(|e| e.to_string())?;
                let mut left = *len;
                while left > 0 {
                    let k = left.min(tmp.len() as u64) as usize;
                    r.read_exact(&mut tmp[..k]).map_err(|e| e.to_string())?;
                    feed(&tmp[..k], &mut buf)?;
                    left -= k as u64;
                }
            }
        }
    }
    if !buf.is_empty() {
        sink(&buf)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn render(p: &Piece, src: &mut Cursor<Vec<u8>>) -> Vec<u8> {
        let mut out = Vec::new();
        stream(p, src, 1000, &mut |d| {
            out.extend_from_slice(d);
            Ok(())
        })
        .unwrap();
        out
    }

    /// Decode a sparse blob back to its expanded image.
    fn unsparse(b: Vec<u8>) -> Vec<u8> {
        let mut c = Cursor::new(b.clone());
        let (blk, total, chunks) = parse_sparse(&mut c).unwrap();
        let mut img = vec![0u8; (blk * total) as usize];
        let mut pos = 0u64;
        for ch in chunks {
            match ch {
                Chunk::Skip(n) => pos += n,
                Chunk::Fill(n, v) => {
                    for i in 0..n * blk / 4 {
                        let o = (pos * blk + i * 4) as usize;
                        img[o..o + 4].copy_from_slice(&v.to_le_bytes());
                    }
                    pos += n;
                }
                Chunk::Raw { off, blocks, .. } => {
                    let (o, l) = ((pos * blk) as usize, (blocks * blk) as usize);
                    img[o..o + l].copy_from_slice(&b[off as usize..off as usize + l]);
                    pos += blocks;
                }
            }
        }
        img
    }

    #[test]
    fn small_image_sent_as_is() {
        let mut c = Cursor::new(vec![1u8; 100]);
        let p = plan(&mut c, Some(1000)).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(render(&p[0], &mut c), vec![1u8; 100]);
    }

    #[test]
    fn raw_split_roundtrips_and_fits() {
        let data: Vec<u8> = (0..(10 * 4096 + 123)).map(|i| (i % 251) as u8).collect();
        let mut c = Cursor::new(data.clone());
        let limit = 3 * 4096 + 100;
        let pieces = plan(&mut c, Some(limit)).unwrap();
        assert!(pieces.len() >= 4);
        let mut merged = vec![0u8; 11 * 4096];
        for p in &pieces {
            assert!(p.len() <= limit);
            let img = unsparse(render(p, &mut c));
            assert_eq!(img.len(), merged.len());
            for (m, i) in merged.iter_mut().zip(img) {
                *m |= i;
            }
        }
        assert_eq!(&merged[..data.len()], &data[..]);
        assert!(merged[data.len()..].iter().all(|b| *b == 0));
    }

    #[test]
    fn sparse_input_is_resplit() {
        // Build sparse: raw 6 blocks, fill 2 blocks, skip 2, raw 1 block.
        let raw: Vec<u8> = (0..7 * 4096).map(|i| (i % 253) as u8 | 1).collect();
        let items = [
            Chunk::Raw {
                off: 0,
                data_len: 6 * 4096,
                blocks: 6,
            },
            Chunk::Fill(2, 0xAABBCCDD),
            Chunk::Skip(2),
            Chunk::Raw {
                off: 6 * 4096,
                data_len: 4096,
                blocks: 1,
            },
        ];
        // Whole sparse file: header, then data laid out via encode().
        let enc = encode(&items, 4096, 11);
        let mut file = Vec::new();
        let mut src = Cursor::new(raw.clone());
        stream(&enc, &mut src, 4096, &mut |d| {
            file.extend_from_slice(d);
            Ok(())
        })
        .unwrap();
        let expect = unsparse(file.clone());
        let mut c = Cursor::new(file);
        let limit = 3 * 4096 + 100;
        let pieces = plan(&mut c, Some(limit)).unwrap();
        assert!(pieces.len() > 1);
        let mut merged = vec![0u8; expect.len()];
        for p in &pieces {
            assert!(p.len() <= limit);
            for (m, i) in merged.iter_mut().zip(unsparse(render(p, &mut c))) {
                *m |= i;
            }
        }
        assert_eq!(merged, expect);
        assert_eq!(expanded_size(&mut c).unwrap(), 11 * 4096);
    }

    #[test]
    fn tiny_limit_errors() {
        let mut c = Cursor::new(vec![1u8; 20000]);
        assert!(plan(&mut c, Some(1000)).is_err());
    }
}
