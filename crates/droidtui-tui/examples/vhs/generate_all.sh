#!/usr/bin/env bash
# Generate all VHS demo GIFs for droidtui
# Usage: ./examples/vhs/generate_all.sh

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if VHS is installed
if ! command -v vhs &> /dev/null; then
    echo -e "${YELLOW}⚠️  VHS is not installed${NC}"
    echo ""
    echo "Install VHS:"
    echo "  macOS:  brew install vhs"
    echo "  Linux:  Download from https://github.com/charmbracelet/vhs/releases"
    echo ""
    exit 1
fi

echo -e "${BLUE}🎬 Generating all VHS demos for droidtui...${NC}"
echo ""

# Change to project root directory
cd "$(dirname "$0")/../.."

# Array of tape files to generate
TAPES=(
    "quickstart"
    "main_menu"
    "full_demo"
    "streaming"
    "device_info"
    "navigation_showcase"
    "package_manager"
    "all_examples"
    "features_highlight"
)

# Generate each demo
for tape in "${TAPES[@]}"; do
    echo -e "${BLUE}📹 Generating ${tape}.gif...${NC}"

    if [ -f "examples/vhs/${tape}.tape" ]; then
        vhs "examples/vhs/${tape}.tape"

        if [ -f "examples/vhs/${tape}.gif" ]; then
            SIZE=$(du -h "examples/vhs/${tape}.gif" | cut -f1)
            echo -e "${GREEN}✅ Generated ${tape}.gif (${SIZE})${NC}"
        else
            echo -e "${YELLOW}⚠️  Failed to generate ${tape}.gif${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Tape file not found: examples/vhs/${tape}.tape${NC}"
    fi

    echo ""
done

echo -e "${GREEN}🎉 All demos generated!${NC}"
echo ""
echo "Generated files:"
for tape in "${TAPES[@]}"; do
    if [ -f "examples/vhs/${tape}.gif" ]; then
        echo "  • examples/vhs/${tape}.gif"
    fi
done
echo ""
echo "To view a demo:"
echo "  open examples/vhs/quickstart.gif"
echo ""
