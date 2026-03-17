---
description: How to manage and deploy the verifyOS-backend service
---

// turbo-all
# Backend Management Workflow

This workflow covers development and deployment for the verifyOS-backend located in `apps/backend`.

## Local Development

1. Navigate to the backend directory:
   `cd apps/backend`

2. Run the server in development mode:
   `cargo run`

3. The server will be available at `http://localhost:7070`.

## Deployment to AWS App Runner

This service is deployed manually using the included deployment script.

1. Ensure your AWS CLI is configured with correct credentials.

2. Run the deployment script from the backend directory:
   `./deploy.sh`

The script will:
- Login to Amazon ECR.
- Build the Docker image for `linux/amd64`.
- Push the image to ECR.
- Trigger a service update on AWS App Runner.
