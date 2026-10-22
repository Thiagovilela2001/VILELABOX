@ECHO OFF
REM Abre o VILELABOX compilado por build-windows.bat com um perfil de teste
REM separado (..\bars-profile-win), como run-bars.sh no Linux.
SETLOCAL
SET "ROOT=%~dp0"
IF NOT EXIST "%ROOT%build-win\mixxx.exe" (ECHO Compile primeiro: build-windows.bat & EXIT /B 1)
REM Sem --resource-path: o Mixxx acha res\ pelo CMakeCache.txt do build-win, com
REM barras "/". Um caminho com "\" quebra as url() das folhas de estilo (QSS).
"%ROOT%build-win\mixxx.exe" --settings-path "%ROOT%..\bars-profile-win" %*
