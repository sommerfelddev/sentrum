0.1.9 (2024-04-30)
------------------
* Improve multsig wallet support and documentation
* Breaking: ntfy action no longer defaults to use "sentrum" as a topic name.
That was highly insecure when used on the ntfy.sh default public server. It will
now default to randomly generating a topic name that will be output in the
terminal. If you were using the old default, you should now look at the terminal
output to retrieve the topic name and add it to your ntfy app.

0.1.8 (2024-04-25)
------------------
* Release aarch64 builds

0.1.7 (2024-04-24)
------------------
* Improvements to release procedure

0.1.6 (2024-04-24)
------------------
* Improved systemd setup and instructions
* Add manpage file in `contrib/man/sentrum.1`

0.1.5 (2024-04-24)
------------------
* Improved action failure logging

0.1.{2,3,4} (2024-04-23)
------------------
* Fixed AUR build
* Added version to release file names
* Sample config and systemd service are now inside binary tarballs instead of as
  separate artifacts
* Fixed missing string in the `electrum.proxy` commented option in the sample
configuration file (#3 by @sethforprivacy)
* Added deployment `Dockerfile` along with CI jobs that keep it updated (#4 by
@sethforprivacy)
* Improved debug logging. Use `RUST_LOG=debug sentrum [...]` to investigate
potential issues

0.1.1 (2024-04-22)
------------------
* Fixed `message.format` deserialization not accepting lowercase values
* Fixed release GH action

0.1.0 (2024-04-21)
------------------
Initial release
