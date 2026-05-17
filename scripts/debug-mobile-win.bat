@echo off
setlocal

if "%~1"=="" (
    echo Usage: %~nx0 [wsl_ip]
    echo Adds portproxy + firewall rules for dev ports 5173, 8080, 1411.
    exit /b 1
)
set WSL_IP=%~1
echo Exposing ports to LAN via WSL_IP=%WSL_IP%

netsh interface portproxy add v4tov4 listenport=5173 listenaddress=0.0.0.0 connectport=5173 connectaddress=%WSL_IP%
netsh advfirewall firewall add rule name="PortProxy 5173" dir=in action=allow protocol=TCP localport=5173

netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=8080 connectaddress=%WSL_IP%
netsh advfirewall firewall add rule name="PortProxy 8080" dir=in action=allow protocol=TCP localport=8080

netsh interface portproxy add v4tov4 listenport=1411 listenaddress=0.0.0.0 connectport=1411 connectaddress=%WSL_IP%
netsh advfirewall firewall add rule name="PortProxy 1411" dir=in action=allow protocol=TCP localport=1411

exit /b 0
