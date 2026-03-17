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

## Deployment

The frontend is built as a standard Next.js application and is currently deployed to Vercel.

1. To create a production build:
   `npm run build`

2. To deploy, push changes to the `main` branch which triggers the Vercel integration, or use the Vercel CLI from `apps/frontend`:
   `vercel --prod`
