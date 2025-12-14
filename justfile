# Justfile for arborium development tasks

# Docker image settings
docker_image := "ghcr.io/bearcove/arborium-plugin-builder"
docker_tag := "latest"

# Build the plugin-builder Docker image (for linux/amd64 - GitHub Actions runners)
docker-build:
    docker build --platform linux/amd64 -t {{docker_image}}:{{docker_tag}} -f .github/docker/Dockerfile.plugin-builder .

# Push the plugin-builder Docker image to GHCR
docker-push: docker-build
    docker push {{docker_image}}:{{docker_tag}}

# Build and push with a specific tag
docker-release tag:
    docker build --platform linux/amd64 -t {{docker_image}}:{{tag}} -f .github/docker/Dockerfile.plugin-builder .
    docker push {{docker_image}}:{{tag}}

# Login to GitHub Container Registry (requires GITHUB_TOKEN env var or gh auth)
docker-login:
    @echo "Logging in to ghcr.io..."
    @gh auth token | docker login ghcr.io -u $(gh api user --jq .login) --password-stdin

grammar-test lang:
    cargo xtask grammar-test {{lang}}
