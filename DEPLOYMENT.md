# Production Deployment Guide for ping0

## Overview
This guide covers deploying ping0 to production using Docker on deploy.cx.

## Prerequisites
- GitHub account (your repo is already set up)
- deploy.cx account
- Domain configured: 0.id.vn

## Environment Variables

Set these environment variables in your deploy.cx dashboard:

```bash
PORT=8080
HOST=0.0.0.0
BASE_URL=https://0.id.vn
RUST_LOG=info
```

## Deployment on deploy.cx

### Method 1: Direct Docker Deployment

1. **Connect Your Repository**
   - Link your GitHub repository (ShayNeeo/ping0) to deploy.cx
   - Select the `main` branch for deployment

2. **Configure Build Settings**
   - Build Command: Auto-detected (uses Dockerfile)
   - Port: 8080
   - Health Check URL: `/health`

3. **Set Environment Variables**
   Add the variables listed above in the deploy.cx dashboard

4. **Configure Persistent Storage**
   - Create a volume mount for `/app/uploads`
   - This ensures uploaded files persist across deployments

5. **Deploy**
   - Click "Deploy" - the build will take 5-10 minutes
   - Monitor logs for any build errors

### Method 2: Using deploy.cx CLI (if available)

```bash
# Install deploy.cx CLI
npm install -g deploycx-cli

# Login
deploycx login

# Deploy
deploycx deploy --env-file .env.example
```

## Post-Deployment Checklist

### 1. Verify Health Check
```bash
curl https://0.id.vn/health
# Expected: {"status":"healthy","service":"ping0"}
```

### 2. Test File Upload
```bash
curl -X POST https://0.id.vn/upload \
  -F "file=@test-image.jpg"
```

### 3. Test QR Generation
```bash
curl -X POST https://0.id.vn/link \
  -d "link=https://example.com"
```

### 4. Monitor Logs
- Check deploy.cx dashboard for application logs
- Look for any errors or warnings
- Verify file uploads are working

## Volume Configuration

**IMPORTANT**: Configure persistent storage for uploads!

### On deploy.cx Dashboard:
1. Go to your service settings
2. Add a volume:
   - **Mount Path**: `/app/uploads`
   - **Storage**: 10GB (adjust based on needs)
   - **Backup**: Enable automatic backups

### Using docker-compose (local testing):
```bash
docker-compose up -d
```

## Monitoring & Maintenance

### Health Checks
- Endpoint: `https://0.id.vn/health`
- Interval: 30 seconds
- Timeout: 10 seconds
- Retries: 3

### Log Levels
Adjust `RUST_LOG` environment variable:
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (recommended)
- `debug` - Detailed debugging
- `trace` - Very verbose

### Resource Limits (Recommended)
- **Memory**: 512MB minimum, 1GB recommended
- **CPU**: 0.5 cores minimum, 1 core recommended
- **Disk**: 10GB for uploads storage

## Security Considerations

✅ **Implemented**:
- Non-root user in Docker container
- File size limits (10MB max)
- File type validation (only: jpg, jpeg, png, gif, webp, pdf, txt)
- URL validation for QR generation
- Health check endpoint
- Proper error handling (no panics)

⚠️ **Recommended Additional Security**:
- Enable HTTPS (should be handled by deploy.cx)
- Add rate limiting (consider using deploy.cx's built-in features)
- Set up automated backups for uploads
- Configure CDN for static assets (optional)
- Add authentication for uploads (if needed)

## Troubleshooting

### Issue: Server not accessible
**Solution**: 
- Verify `HOST=0.0.0.0` is set
- Check port configuration matches deploy.cx settings
- Review firewall rules

### Issue: Health check failing
**Solution**:
- Verify `/health` endpoint returns 200
- Check server logs for startup errors
- Ensure sufficient memory allocated

### Issue: File uploads failing
**Solution**:
- Check volume mount configuration
- Verify `/app/uploads` directory permissions
- Check disk space availability
- Review file size limits (10MB max)

### Issue: Images not loading
**Solution**:
- Verify `BASE_URL` environment variable is correct
- Check `/files/*` route is working
- Ensure uploaded files are in `/app/uploads` directory

## Rollback Procedure

If deployment fails:

1. **Via deploy.cx Dashboard**:
   - Go to Deployments history
   - Click "Rollback" on previous working version

2. **Via Git**:
   ```bash
   git revert HEAD
   git push origin main
   ```

## Performance Optimization

### Current Configuration
- Rust release builds with optimizations
- Minimal Docker image (debian:bullseye-slim)
- Efficient WASM frontend

### Scaling Recommendations
- **Horizontal Scaling**: Deploy multiple instances behind a load balancer
- **CDN**: Use Cloudflare or similar for static assets
- **Database**: If adding user accounts, consider PostgreSQL
- **Caching**: Add Redis for frequently accessed data

## Maintenance Tasks

### Weekly
- Review error logs
- Check disk usage for uploads
- Monitor response times

### Monthly
- Update dependencies (security patches)
- Review and clean old uploads if needed
- Check backup integrity

### Quarterly
- Review and update security policies
- Performance testing
- Dependency updates

## Support & Resources

- **Repository**: https://github.com/ShayNeeo/ping0
- **Domain**: https://0.id.vn
- **Health Check**: https://0.id.vn/health

## Quick Commands Reference

```bash
# Check health
curl https://0.id.vn/health

# Upload file
curl -X POST https://0.id.vn/upload -F "file=@image.jpg"

# Generate QR
curl -X POST https://0.id.vn/link -d "link=https://example.com"

# View logs (deploy.cx)
deploycx logs --follow

# Restart service (deploy.cx)
deploycx restart
```

## Notes for deploy.cx

- The Dockerfile pulls from GitHub automatically
- Build time: 5-10 minutes (Rust compilation is slow)
- Consider enabling build caching if available
- The app is stateless except for uploaded files (requires volume)
