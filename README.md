# kerio-connect-reindex

Kerio Connect mail server tool for reindexing all index.fld and search.fld on Linux

## Enable the timer on your server

When you install the newly built .deb or .rpm package, the files will be placed in the correct directories automatically. However, systemd timers usually need to be explicitly enabled to start running.

After installing the package on your server, run these two commands to activate the schedule:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kerio-connect-reindex.timer
```

You can verify that the timer is actively waiting for its next execution date by running:

```bash
systemctl list-timers | grep kerio
```
