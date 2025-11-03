# 📛 How to Update README Badges

After you push to GitHub, update the badges in README.md with your actual GitHub username.

## 🔄 Quick Update

Replace `YOUR_USERNAME` with your actual GitHub username in these lines:

### In README.md (Line 18-22)

**Find:**
```markdown
[![CI](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/BLAZE/actions)
[![Build](https://img.shields.io/badge/Build-Auto--Tested-32CD32?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/YOUR_USERNAME/BLAZE/actions)
[![Coverage](https://img.shields.io/badge/Coverage-100%25-00D4FF?style=for-the-badge&logo=codecov&logoColor=black)](https://github.com/YOUR_USERNAME/BLAZE)
[![Performance](https://img.shields.io/badge/Speed-3x+Faster-FF6B35?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/YOUR_USERNAME/BLAZE)
```

**Replace with** (example for username `johndoe`):
```markdown
[![CI](https://github.com/johndoe/BLAZE/workflows/CI/badge.svg)](https://github.com/johndoe/BLAZE/actions)
[![Build](https://img.shields.io/badge/Build-Auto--Tested-32CD32?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/johndoe/BLAZE/actions)
[![Coverage](https://img.shields.io/badge/Coverage-100%25-00D4FF?style=for-the-badge&logo=codecov&logoColor=black)](https://github.com/johndoe/BLAZE)
[![Performance](https://img.shields.io/badge/Speed-3x+Faster-FF6B35?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/johndoe/BLAZE)
```

## 📝 Or Use Find & Replace

In your editor:
1. Open `README.md`
2. Find: `YOUR_USERNAME`
3. Replace with: `your-actual-github-username`
4. Replace All

## ✨ Additional Badges (Optional)

You can also add these advanced badges:

### GitHub Actions Status
```markdown
![CI](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg?branch=main)
```

### Issues Badge
```markdown
[![Issues](https://img.shields.io/github/issues/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE/issues)
```

### Pull Requests Badge
```markdown
[![PRs](https://img.shields.io/github/issues-pr/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE/pulls)
```

### Contributors Badge
```markdown
[![Contributors](https://img.shields.io/github/contributors/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE/graphs/contributors)
```

### Last Commit Badge
```markdown
[![Last Commit](https://img.shields.io/github/last-commit/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE/commits/main)
```

### Release Badge
```markdown
[![Release](https://img.shields.io/github/v/release/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE/releases)
```

### Code Size Badge
```markdown
[![Code Size](https://img.shields.io/github/languages/code-size/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE)
```

### Top Language Badge
```markdown
[![Language](https://img.shields.io/github/languages/top/YOUR_USERNAME/BLAZE)](https://github.com/YOUR_USERNAME/BLAZE)
```

## 🎨 Complete Badge Section Example

```markdown
<div align="center">

<!-- Main Badges -->
[![BLAZE](https://img.shields.io/badge/BLAZE-v1.0.0-FF6B35?style=for-the-badge&logo=fire&logoColor=white)](https://github.com/YOUR_USERNAME/BLAZE)
[![License](https://img.shields.io/badge/License-MIT-8B5CF6?style=for-the-badge&logo=balance-scale&logoColor=white)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Cross--Platform-10F5CC?style=for-the-badge&logo=globe&logoColor=black)](https://github.com/YOUR_USERNAME/BLAZE)
[![Stars](https://img.shields.io/github/stars/YOUR_USERNAME/BLAZE?style=for-the-badge&color=FFD700&logo=github)](https://github.com/YOUR_USERNAME/BLAZE/stargazers)

<!-- CI/CD Badges -->
[![CI](https://github.com/YOUR_USERNAME/BLAZE/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/BLAZE/actions)
[![Build](https://img.shields.io/badge/Build-Auto--Tested-32CD32?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/YOUR_USERNAME/BLAZE/actions)
[![Tests](https://img.shields.io/badge/Tests-Passing-32CD32?style=for-the-badge&logo=check-circle&logoColor=white)](https://github.com/YOUR_USERNAME/BLAZE/actions)
[![LLVM](https://img.shields.io/badge/LLVM-15.0-00D4FF?style=for-the-badge&logo=llvm&logoColor=black)](https://llvm.org)

<!-- Stats Badges -->
[![Issues](https://img.shields.io/github/issues/YOUR_USERNAME/BLAZE?style=flat-square)](https://github.com/YOUR_USERNAME/BLAZE/issues)
[![PRs](https://img.shields.io/github/issues-pr/YOUR_USERNAME/BLAZE?style=flat-square)](https://github.com/YOUR_USERNAME/BLAZE/pulls)
[![Contributors](https://img.shields.io/github/contributors/YOUR_USERNAME/BLAZE?style=flat-square)](https://github.com/YOUR_USERNAME/BLAZE/graphs/contributors)
[![Last Commit](https://img.shields.io/github/last-commit/YOUR_USERNAME/BLAZE?style=flat-square)](https://github.com/YOUR_USERNAME/BLAZE/commits/main)

</div>
```

## 🔗 Badge Resources

- **Shields.io**: https://shields.io/ (custom badges)
- **GitHub Badges**: https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/adding-a-workflow-status-badge
- **Badge Generator**: https://michaelcurrin.github.io/badge-generator/

## ✅ After Updating

1. Commit changes:
   ```bash
   git add README.md
   git commit -m "docs: Update badges with correct username"
   git push origin main
   ```

2. Check your README on GitHub
3. Badges should now show live status!

---

**Note**: Badges may take 1-2 minutes to update after your first push.
