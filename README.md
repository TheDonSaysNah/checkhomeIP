# CheckHomeIP

A small and lightweight program to monitor your home IP 24/7 and notify yourself by either sending an email using SMTP or using [NTFY](https://ntfy.sh). Useful for people who don't have DDNS and need to know if their ISP assigned IP has been changed.

## Getting started
To get started, fill out `checkip.env` with the SMTP or NTFY fields you wish to use. Compile and run.

*This has only been tested with GMail SMTP. Other hosts may not work.*

### Systemd service setup
To enable systemd service automation, edit `checkip.service` and change the lines containing `ExecStart=` and `WorkingDirectory=` to where the binary is in your filesystem. Be sure to put the folder path and not binary path for `WorkingDirectory=`

Once done, change the line `User=user` to the user account you wish the service to run as and copy/move the file to `/etc/systemd/system/`

Finally, enable the service and start it using `systemctl enable --now checkip.service`


### New client env arg
A new env arg has been introduced and is enabled with -c/--client.
The purpose of this was mainly for myself as I have 2 different networks across my home country so by using ``-c`` I can determine which network had the IP changed.

Example: ``./checkhomeip -nc parents``
- ``-n`` to notify using [NTFY](https://ntfy.sh) service.
- ``-c parents`` all notifications will be titled with "parents" and also gives the local IP. Useful if you are unable to set a static local IP and need to SSH. 


