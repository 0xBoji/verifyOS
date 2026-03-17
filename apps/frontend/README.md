# verifyOS Frontend 🚀

The verifyOS web frontend is a premium, iOS-inspired diagnostic dashboard built with Next.js. it provides a seamless interface for security researchers and developers to analyze Apple bundles and Xcode projects with zero cloud transmission.

## ✨ Core Features

- **Premium Design System**: iOS-inspired glassmorphism, custom scrollbars, and smooth micro-animations.
- **Smart Folder Discovery**: Automatically identifies scannable targets (`.app`, `.xcodeproj`, `.xcworkspace`) within uploaded folders.
- **Diagnostic AST Mode**: Interactive tree visualization of findings with Pan & Zoom capabilities (Google Maps style).
- **In-Browser Bundling**: Client-side folder zipping using `JSZip` to handle complex Xcode project structures without server-side intermediate storage.
- **Agent Handoff**: One-click generation of AI Agent Packs for automated remediation.

## 🛠️ Technical Stack

- **Framework**: [Next.js](https://nextjs.org/) (App Router)
- **Compiler**: [Turbopack](https://nextjs.org/docs/app/api-reference/turbopack)
- **Styling**: Vanilla CSS with iOS-inspired design tokens.
- **Utilities**: `JSZip` for client-side compression, `react-icons` for unified iconography.

## 🚀 Getting Started

### Prerequisites

Ensure you have [Node.js](https://nodejs.org/) and `pnpm` (or `npm`) installed.

### Installation

```bash
cd apps/frontend
npm install
```

### Development

```bash
npm run dev
```

The app will be available at [http://localhost:3000](http://localhost:3000).

### Backend Integration

The frontend expects the verifyOS backend to be running on `http://127.0.0.1:7070`.

To start the backend:
```bash
cargo run --manifest-path apps/backend/Cargo.toml
```

## 📂 Project Structure

- `src/app/page.tsx`: Main dashboard and logic.
- `src/app/globals.css`: Design system and component styles.
- `public/`: Static assets and iconography.

## 🌈 Design Philosophy

The UI is built on a **"Glassmorphic macOS/iOS"** aesthetic:
- **Translucency**: High use of `backdrop-filter: blur()`.
- **Soft Shadows**: Layered shadows to create depth.
- **Interactive Feedback**: Smooth transitions and scale transforms on hover/click.
- **Legibility**: Clean typography and generous white space.
