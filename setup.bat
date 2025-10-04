@echo off
title BLAZE COMPILER SETUP - by BLACK
color 0A
cls

echo.
echo  ██████╗ ██╗      █████╗ ███████╗███████╗    ██████╗ ██████╗ ███╗   ███╗██████╗ ██╗██╗     ███████╗███████╗
echo  ██╔══██╗██║     ██╔══██╗╚══███╔╝╚══███╔╝   ██╔════╝██╔═══██╗████╗ ████║██╔══██╗██║██║     ██╔════╝██╔════╝
echo  ██████╔╝██║     ███████║  ███╔╝   ███╔╝    ██║     ██║   ██║██╔████╔██║██████╔╝██║██║     █████╗  ███████╗
echo  ██╔══██╗██║     ██╔══██║ ███╔╝   ███╔╝     ██║     ██║   ██║██║╚██╔╝██║██╔═══╝ ██║██║     ██╔══╝  ╚════██║
echo  ██████╔╝███████╗██║  ██║███████╗███████╗   ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║     ██║███████╗███████╗███████║
echo  ╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝    ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚══════╝╚══════╝
echo.
echo  ███████╗███████╗████████╗██╗   ██╗██████╗ 
echo  ██╔════╝██╔════╝╚══██╔══╝██║   ██║██╔══██╗
echo  ███████╗█████╗     ██║   ██║   ██║██████╔╝
echo  ╚════██║██╔══╝     ██║   ██║   ██║██╔═══╝ 
echo  ███████║███████╗   ██║   ╚██████╔╝██║     
echo  ╚══════╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝     
echo.
echo  ╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗
echo  ║                                    🚀 BLAZE COMPILER SETUP 🚀                                              ║
echo  ║                                                                                                              ║
echo  ║  Developed by: BLACK (https://github.com/BLACK0X80)                                                           ║
echo  ║  Version: 1.0.0                                                                                             ║
echo  ║  A powerful systems programming language with Rust-like safety guarantees                                   ║
echo  ║                                                                                                              ║
echo  ║  Features:                                                                                                    ║
echo  ║  • Fast compilation                                                                                         ║
echo  ║  • Memory safety                                                                                            ║
echo  ║  • Zero-cost abstractions                                                                                   ║
echo  ║  • Modern syntax                                                                                             ║
echo  ║  • Cross-platform support                                                                                   ║
echo  ╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝
echo.

echo [INFO] Starting BLAZE Compiler installation...
echo [INFO] Developer: BLACK
echo [INFO] Version: 1.0.0
echo.

echo [STEP 1/5] Building BLAZE Compiler...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build BLAZE Compiler!
    pause
    exit /b 1
)
echo [SUCCESS] BLAZE Compiler built successfully!

echo.
echo [STEP 2/5] Creating installation directory...
if not exist "C:\Program Files\BLAZE" mkdir "C:\Program Files\BLAZE"
if not exist "C:\Program Files\BLAZE\bin" mkdir "C:\Program Files\BLAZE\bin"
if not exist "C:\Program Files\BLAZE\examples" mkdir "C:\Program Files\BLAZE\examples"
echo [SUCCESS] Installation directory created!

echo.
echo [STEP 3/5] Copying BLAZE Compiler files...
copy "target\release\blaze.exe" "C:\Program Files\BLAZE\bin\blaze.exe" >nul
copy "examples\*.blz" "C:\Program Files\BLAZE\examples\" >nul
echo [SUCCESS] Files copied successfully!

echo.
echo [STEP 4/5] Adding BLAZE to system PATH...
setx PATH "%PATH%;C:\Program Files\BLAZE\bin" /M >nul
if %errorlevel% neq 0 (
    echo [WARNING] Failed to add to system PATH. Please run as administrator!
    echo [INFO] You can manually add C:\Program Files\BLAZE\bin to your PATH
) else (
    echo [SUCCESS] BLAZE added to system PATH!
)

echo.
echo [STEP 5/5] Creating desktop shortcut...
set "desktop=%USERPROFILE%\Desktop"
echo [InternetShortcut] > "%desktop%\BLAZE Compiler.url"
echo URL=file:///C:/Program Files/BLAZE/bin/blaze.exe >> "%desktop%\BLAZE Compiler.url"
echo IconFile=C:\Program Files\BLAZE\bin\blaze.exe >> "%desktop%\BLAZE Compiler.url"
echo IconIndex=0 >> "%desktop%\BLAZE Compiler.url"
echo [SUCCESS] Desktop shortcut created!

echo.
echo  ╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗
echo  ║                                    🎉 INSTALLATION COMPLETE! 🎉                                          ║
echo  ║                                                                                                              ║
echo  ║  BLAZE Compiler has been successfully installed!                                                             ║
echo  ║                                                                                                              ║
echo  ║  Installation Path: C:\Program Files\BLAZE\                                                                 ║
echo  ║  Examples Path: C:\Program Files\BLAZE\examples\                                                             ║
echo  ║                                                                                                              ║
echo  ║  Usage Examples:                                                                                            ║
echo  ║  • blaze check example.blz                                                                                 ║
echo  ║  • blaze build example.blz                                                                                 ║
echo  ║  • blaze run example.blz                                                                                   ║
echo  ║                                                                                                              ║
echo  ║  Thank you for using BLAZE Compiler!                                                                        ║
echo  ║  Developed by: BLACK (https://github.com/BLACK0X80)                                                         ║
echo  ╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝
echo.

echo [INFO] Testing BLAZE Compiler installation...
"C:\Program Files\BLAZE\bin\blaze.exe" check "C:\Program Files\BLAZE\examples\hello.blz"
if %errorlevel% equ 0 (
    echo [SUCCESS] BLAZE Compiler is working correctly!
) else (
    echo [WARNING] BLAZE Compiler test failed. Please check installation.
)

echo.
echo [INFO] Installation completed successfully!
echo [INFO] You may need to restart your command prompt for PATH changes to take effect.
echo.
echo Press any key to exit...
pause >nul
