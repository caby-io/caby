@echo off
setlocal

echo Removing portproxy + firewall rules for dev ports (5173, 8080, 1411)

netsh interface portproxy delete v4tov4 listenport=5173 listenaddress=0.0.0.0
netsh advfirewall firewall delete rule name="PortProxy 5173"

netsh interface portproxy delete v4tov4 listenport=8080 listenaddress=0.0.0.0
netsh advfirewall firewall delete rule name="PortProxy 8080"

netsh interface portproxy delete v4tov4 listenport=1411 listenaddress=0.0.0.0
netsh advfirewall firewall delete rule name="PortProxy 1411"

exit /b 0
