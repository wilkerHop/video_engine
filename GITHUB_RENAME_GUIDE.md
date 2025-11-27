# GitHub Repository Rename Guide

## Renaming Your Repository on GitHub

Since the repository folder is currently `video_engine` but the package is named `interstellar-triangulum`, here's how to align everything:

### Option 1: Rename on GitHub (Recommended)

1. **Go to your GitHub repository**
2. **Click "Settings"** tab
3. **Scroll to "Repository name"** section
4. **Change name to**: `interstellar-triangulum`
5. **Click "Rename"**

GitHub will automatically:
- Redirect old URLs to new name
- Update clone URLs
- Preserve issues, PRs, stars, etc.

### Option 2: Update Local Remote

After renaming on GitHub, update your local remote:

```bash
cd /Users/wilkerribeiro/.gemini/antigravity/playground/interstellar-triangulum/video_engine

# Update remote URL (replace YOUR_USERNAME with your GitHub username)
git remote set-url origin https://github.com/YOUR_USERNAME/interstellar-triangulum.git

# Verify
git remote -v
```

### Option 3: Rename Local Folder (Optional)

If you want to rename the local folder from `video_engine` to `interstellar-triangulum`:

```bash
cd /Users/wilkerribeiro/.gemini/antigravity/playground/interstellar-triangulum
mv video_engine interstellar-triangulum
cd interstellar-triangulum
```

---

## Documentation Already Updated

✅ **Cargo.toml**: Package name is `interstellar-triangulum`  
✅ **README.md**: Project title is "Interstellar Triangulum"  
✅ **CONTRIBUTING.md**: Project structure reflects new name  
✅ **Source code**: All imports use `interstellar_triangulum`  

## What Needs Your GitHub Username

The README badges currently have `YOUR_USERNAME` placeholder:

```markdown
[![CI](https://github.com/YOUR_USERNAME/interstellar-triangulum/workflows/CI/badge.svg)]
```

Replace `YOUR_USERNAME` with your actual GitHub username.

---

**Next Steps:**
1. Rename repository on GitHub
2. Update README badges with your username
3. Push changes
