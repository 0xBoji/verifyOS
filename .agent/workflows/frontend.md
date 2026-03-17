---
description: How to manage and deploy the verifyOS-frontend (Web App)
---

// turbo-all
# Frontend Management Workflow

This workflow covers development and deployment for the verifyOS-frontend located in `apps/frontend`.

## Local Development

1. Navigate to the frontend directory:
   `cd apps/frontend`

2. Install dependencies (if needed):
   `npm install`

3. Run the development server:
   `npm run dev`

4. Access the app at `http://localhost:3000`.

## Diagnostic Workflows

### Verifying AST Visualization
1. Open the app and load the **Example Report**.
2. Locate a finding in the list and click **Draw** or **View in AST** (if available).
3. Ensure the `ASTModal` opens and centers the selected node.
4. Verify Pan & Zoom:
   - **Pan**: Drag the background.
   - **Zoom**: Cmd/Ctrl + Scroll or use the zoom controls.

### Testing Folder Discovery
1. Click **Choose folder** in the Quick Scan panel.
2. Select a directory containing Apple targets (e.g., `examples/AppStoreMock`).
3. Verify that the "Auto-discovered scannable items" section appears with the correct targets.
4. Verify that clicking a target triggers a client-side bundle and sets it for scanning.

## Deployment

The frontend is built as a standard Next.js application and is currently deployed to Vercel.

1. To create a production build:
   `npm run build`

2. To deploy, push changes to the `main` branch which triggers the Vercel integration, or use the Vercel CLI from `apps/frontend`:
   `vercel --prod`
