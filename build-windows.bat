@ECHO OFF
REM Compila o VILELABOX no Windows: MSVC (Visual Studio 2022) + ambiente de
REM dependências pré-compilado do Mixxx (tools\windows_buildenv.bat).
REM Saída: build-win\mixxx.exe. Testes desligados: alguns usam /tmp e ::kill.
REM Uso: build-windows.bat
SETLOCAL
SET "ROOT=%~dp0"

REM cmake/ninja instalados com "pip install --user" ficam fora do PATH.
WHERE cmake >NUL 2>&1 || SET "PATH=%APPDATA%\Python\Python313\Scripts;%APPDATA%\Python\Python312\Scripts;%PATH%"
WHERE cmake >NUL 2>&1 || (ECHO Erro: cmake nao encontrado. Instale com: python -m pip install --user cmake ninja & EXIT /B 1)

SET "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
FOR /F "usebackq delims=" %%I IN (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) DO SET "VSPATH=%%I"
IF NOT DEFINED VSPATH (ECHO Erro: Visual Studio com C++ nao encontrado. & EXIT /B 1)
CALL "%VSPATH%\VC\Auxiliary\Build\vcvars64.bat" >NUL || EXIT /B 1

CALL "%ROOT%tools\windows_buildenv.bat" setup || EXIT /B 1

cmake -S "%ROOT%." -B "%ROOT%build-win" -G Ninja ^
    -DCMAKE_BUILD_TYPE=RelWithDebInfo ^
    -DBUILD_TESTING=OFF ^
    -DBUILD_BENCH=OFF ^
    -DMIXXX_VCPKG_ROOT="%MIXXX_VCPKG_ROOT%" || EXIT /B 1
cmake --build "%ROOT%build-win" --parallel || EXIT /B 1

ECHO.
ECHO Pronto: %ROOT%build-win\mixxx.exe
