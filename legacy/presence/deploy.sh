#!/bin/bash
# Deploy presence server to Cloud Run
# Usage: ./presence/deploy.sh

set -e

cd "$(dirname "$0")"

PROJECT_ID="prismdatafeed"
SERVICE_NAME="jarvis-presence"
REGION="us-central1"
IMAGE="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

echo "Building Docker image..."
docker build --platform linux/amd64 -t $IMAGE .

echo "Pushing to GCR..."
docker push $IMAGE

echo "Deploying to Cloud Run..."
gcloud run deploy $SERVICE_NAME \
    --image $IMAGE \
    --region $REGION \
    --memory 256Mi \
    --cpu 1 \
    --timeout 3600 \
    --max-instances 1 \
    --min-instances 1 \
    --allow-unauthenticated \
    --session-affinity \
    --project $PROJECT_ID

# Get the service URL
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region $REGION --project $PROJECT_ID --format 'value(status.url)')

# Cloud Run gives https:// URL, WebSocket uses wss://
WS_URL=$(echo "$SERVICE_URL" | sed 's|https://|wss://|')

echo ""
echo "Deployed!"
echo "  Service URL: $SERVICE_URL"
echo "  WebSocket:   $WS_URL"
echo ""
echo "Set in your .env:"
echo "  PRESENCE_URL=$WS_URL"
