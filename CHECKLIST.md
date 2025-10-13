# Pre-Deployment Checklist

Before deploying to production, verify the following:

## ✅ Code Changes
- [x] Server binds to 0.0.0.0 (not 127.0.0.1)
- [x] Environment variables configured
- [x] Health check endpoint added
- [x] Error handling (no unwrap() calls)
- [x] File size limits implemented
- [x] File type validation added
- [x] URL validation for QR codes
- [x] Logging configured

## ✅ Docker Configuration
- [x] Dockerfile optimized
- [x] Non-root user created
- [x] Health check configured
- [x] .dockerignore file created
- [x] Volume support for uploads

## ✅ Documentation
- [x] DEPLOYMENT.md created
- [x] README.md updated
- [x] PRODUCTION_REPORT.md created
- [x] .env.example provided
- [x] docker-compose.yml for local testing

## 📋 Deploy.cx Configuration

### Before Deploying:
- [ ] Repository connected to deploy.cx
- [ ] Branch selected (main)
- [ ] Environment variables set:
  - [ ] BASE_URL=https://0.id.vn
  - [ ] PORT=8080
  - [ ] HOST=0.0.0.0
  - [ ] RUST_LOG=info
- [ ] Persistent volume configured:
  - [ ] Mount path: /app/uploads
  - [ ] Size: 10GB+
  - [ ] Backups enabled
- [ ] Health check configured:
  - [ ] Path: /health
  - [ ] Interval: 30s
  - [ ] Timeout: 10s

### After Deploying:
- [ ] Deployment successful (no errors)
- [ ] Health check passing
- [ ] Test file upload works
- [ ] Test QR generation works
- [ ] Check logs for errors
- [ ] Verify domain points to service
- [ ] SSL/HTTPS working
- [ ] Monitor resource usage

## 🧪 Testing

### Local Testing (if Docker available):
```bash
# Build
docker build -t ping0 .

# Run
docker run -p 8080:8080 \
  -e BASE_URL=http://localhost:8080 \
  -v $(pwd)/uploads:/app/uploads \
  ping0

# Test health
curl http://localhost:8080/health

# Test upload
curl -X POST http://localhost:8080/upload \
  -F "file=@test-image.jpg"

# Test QR
curl -X POST http://localhost:8080/link \
  -d "link=https://example.com"
```

### Production Testing:
```bash
# Health check
curl https://0.id.vn/health

# Upload test
curl -X POST https://0.id.vn/upload \
  -F "file=@test-image.jpg"

# QR generation test
curl -X POST https://0.id.vn/link \
  -d "link=https://example.com"
```

## 🔐 Security Checklist
- [x] Non-root user in Docker
- [x] File size limits (10MB)
- [x] File type validation
- [x] URL validation
- [x] No hardcoded secrets
- [x] Environment-based config
- [ ] HTTPS enabled (deploy.cx)
- [ ] Rate limiting (deploy.cx)
- [ ] Regular backups configured

## 📊 Monitoring Setup
- [ ] Health check alerts configured
- [ ] Log monitoring enabled
- [ ] Resource usage alerts set
- [ ] Backup verification scheduled
- [ ] Error tracking enabled

## 🚀 Ready to Deploy?

If all checkboxes above are completed, you're ready to deploy!

```bash
# 1. Commit changes
git add .
git commit -m "Production-ready configuration"
git push origin main

# 2. Deploy on deploy.cx
# - Follow instructions in DEPLOYMENT.md
# - Monitor the build process
# - Verify health check

# 3. Test in production
# - Run all tests listed above
# - Monitor logs
# - Verify functionality

# 4. Go live!
```

## 📞 Need Help?

1. Review DEPLOYMENT.md for detailed instructions
2. Check PRODUCTION_REPORT.md for changes made
3. Review deploy.cx documentation
4. Check application logs

## 🎉 Post-Launch

After successful deployment:
- Share the URL: https://0.id.vn
- Monitor for first 24-48 hours
- Set up regular maintenance schedule
- Plan for future enhancements
