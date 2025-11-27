# PDF Password Recovery Tool

Windows GUI application for recovering PDF passwords using client initials and SSN patterns.

## ðŸš€ Quick Start

1. Download latest release
2. Extract both .exe files to same folder
3. Run pdfrip-gui.exe
4. Enter client information
5. Click "Start Recovery"

## ðŸ“‹ Password Patterns

For initials **T.C.** and SSN **1234**, tests:

- Numbers: 0000-9999, 00000-99999, 000000-999999
- tc0000-tc999999 (lowercase)
- TC0000-TC999999 (uppercase)  
- Tc0000-Tc999999 (title case)
- tC0000-tC999999 (inverse)

Total: 15 patterns automatically tested

## âš¡ System Requirements

- Windows 10/11
- 8GB RAM recommended
- Multi-core CPU for best performance

## ðŸ”§ Building

```bash
# Install Rust from https://rustup.rs/
cargo build --release --bin pdfrip
cargo build --release --bin pdfrip-gui
```

## âš–ï¸ Attribution

Based on **PDFRip v2.0.1** by Mufeed VH & Pommaq
- Repository: https://github.com/mufeedvh/pdfrip

Customized for [YOUR COMPANY NAME]

## ðŸ“„ License

See original PDFRip license. For internal use only.

## âš ï¸ Legal Notice

Only use on PDFs you're authorized to access. Unauthorized use is prohibited.
