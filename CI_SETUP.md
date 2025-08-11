# CI Setup Guide for Building and Publishing Wheels

This guide explains how to set up GitHub CI to build wheels and publish to PyPI.

## Step 1: Enable GitHub Actions

1. Push your code to GitHub
2. Go to your repository on GitHub
3. Click on the "Actions" tab
4. GitHub Actions should be enabled by default. If not, click "Enable Actions"

## Step 2: Set Up PyPI Publishing (Trusted Publishing - Recommended)

### 2.1 Create PyPI Account
1. Go to https://pypi.org and create an account
2. Verify your email address

### 2.2 Configure Trusted Publishing on PyPI
1. Log in to PyPI
2. Go to your account settings → Publishing
3. Add a new trusted publisher:
   - Publisher: GitHub
   - Repository owner: `<your-github-username>`
   - Repository name: `sup` (or your repo name)
   - Workflow name: `build.yml`
   - Environment name: `pypi` (matches the workflow)
4. Save the configuration

### 2.3 Test with TestPyPI (Optional but Recommended)
1. Create an account on https://test.pypi.org
2. Configure trusted publishing the same way as above
3. Test your release process with TestPyPI first

## Step 3: Build Your First Wheels

### Option A: Manual Trigger (Testing)
1. Go to Actions tab in your GitHub repo
2. Select "Build and Release" workflow
3. Click "Run workflow"
4. Select options:
   - Branch: `main` or `master`
   - Publish to PyPI: `none` (just build), `test` (TestPyPI), or `prod` (PyPI)
5. Click "Run workflow"

### Option B: Push to Branch (CI Build)
```bash
git add .
git commit -m "Add CI configuration"
git push origin main
```
This will trigger a build but won't publish to PyPI.

### Option C: Create a Release (Full Release)
```bash
# Tag your release
git tag -a v0.1.0 -m "First release"
git push origin v0.1.0
```
This will:
- Build wheels for all platforms
- Publish to PyPI automatically (if trusted publishing is configured)

For release candidates (won't publish to prod PyPI):
```bash
git tag -a v0.1.0-rc1 -m "Release candidate"
git push origin v0.1.0-rc1
```

## Step 4: Monitor the Build

1. Go to the Actions tab
2. Click on the running workflow
3. Watch the build progress for each platform:
   - Linux (x86_64, aarch64)
   - Windows (x64, x86)
   - macOS (x86_64, ARM64)
   - Source distribution (sdist)

## Step 5: Download Built Wheels (Without Publishing)

If you just want to build wheels without publishing:
1. Let the workflow complete
2. Go to the workflow run page
3. Scroll down to "Artifacts"
4. Download the wheel files for each platform

## Alternative: API Token Method (Less Secure)

If you prefer using API tokens instead of trusted publishing:

1. Generate an API token on PyPI:
   - Go to Account Settings → API tokens
   - Create a new token (scope: entire account or specific project)

2. Add the token to GitHub Secrets:
   - Go to repo Settings → Secrets and variables → Actions
   - Add new secret: `PYPI_API_TOKEN`
   - Paste your token

3. Modify `.github/workflows/build.yml` release step:
```yaml
- name: Publish to PyPI
  uses: pypa/gh-action-pypi-publish@release/v1
  with:
    password: ${{ secrets.PYPI_API_TOKEN }}
```

## Workflow Features

The CI workflow includes:
- **Multi-platform builds**: Linux, Windows, macOS (both x86_64 and ARM)
- **Python 3.13 support**: Builds specifically for Python 3.13
- **Automatic ripgrep compilation**: Builds ripgrep from source for each platform
- **Manual dispatch**: Trigger builds manually with custom options
- **Tag-based releases**: Automatically publish when you push a version tag
- **TestPyPI support**: Test releases with `-rc` tags
- **Artifact uploads**: All wheels are saved as GitHub artifacts

## Troubleshooting

### Build Fails
- Check the logs in the Actions tab
- Common issues:
  - Missing Rust toolchain (should be auto-installed)
  - Network issues downloading ripgrep source
  - Platform-specific compilation errors

### Publishing Fails
- Verify trusted publishing is configured correctly
- Check the package name isn't already taken on PyPI
- Ensure version number in `pyproject.toml` is updated
- For first-time publishing, you may need to manually create the project on PyPI first

### Testing Locally
Before pushing, test the build locally:
```bash
make clean
make build
make test
maturin build --release  # Build release wheel locally
```

## Version Management

Update version in `pyproject.toml` before releasing:
```toml
[project]
name = "sup"
version = "0.1.1"  # Increment this
```

Then tag and push:
```bash
git add pyproject.toml
git commit -m "Bump version to 0.1.1"
git tag -a v0.1.1 -m "Release version 0.1.1"
git push origin main
git push origin v0.1.1
```