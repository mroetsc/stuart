# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [0.2.0](https://github.com/mroetsc/stuart/compare/1cc1082531fa33f396e281c51fd12d336d519e3a..0.2.0) - 2026-06-16
#### Features
- (**cli**) config value priority and args for creating new default config - ([0f87016](https://github.com/mroetsc/stuart/commit/0f870168121c02fec5ba9bd1c7bf49e192dd1b55)) - [@mroetsc](https://github.com/mroetsc)
- (**config**) implemented first draft of config module - ([4911c73](https://github.com/mroetsc/stuart/commit/4911c73334825e433b90c7220f43e44405d5cd54)) - [@mroetsc](https://github.com/mroetsc)
#### Documentation
- updated readme with config file info - ([3bbe66d](https://github.com/mroetsc/stuart/commit/3bbe66dee842c6cf353b479ccd5bc83baaf17923)) - [@mroetsc](https://github.com/mroetsc)
#### Build system
- configured release profile and set config crate feature flags to reduce binary size - ([2972eb5](https://github.com/mroetsc/stuart/commit/2972eb51d13c44585d10fa9a5f68b7b6f51496ea)) - [@mroetsc](https://github.com/mroetsc)
#### Refactoring
- (**cli**) used From impls for mapping Arg structs more cleanly - ([09b0c9e](https://github.com/mroetsc/stuart/commit/09b0c9ed3053971931f5c5b16cd58a8f79c76afe)) - [@mroetsc](https://github.com/mroetsc)
- (**config**) improve validation, error handling, and cross-platform path resolution - ([4fef885](https://github.com/mroetsc/stuart/commit/4fef8857abd816ddba3a9ef55bf0a25b5d94f535)) - [@mroetsc](https://github.com/mroetsc)
#### Miscellaneous Chores
- (**cli**) minor spelling changes - ([17cef55](https://github.com/mroetsc/stuart/commit/17cef55e99a90d8f546444b06b4092088ce84b62)) - [@mroetsc](https://github.com/mroetsc)
- (**docs**) changed help output highlighting in readme - ([1cc1082](https://github.com/mroetsc/stuart/commit/1cc1082531fa33f396e281c51fd12d336d519e3a)) - [@mroetsc](https://github.com/mroetsc)
- dded dirs dependency - ([95eba5d](https://github.com/mroetsc/stuart/commit/95eba5d3c1f625d5ee40e1a5f57c638acc0cc6ea)) - [@mroetsc](https://github.com/mroetsc)
- added serde and config depedency - ([10d5b32](https://github.com/mroetsc/stuart/commit/10d5b327fb55a0693a41f9641696ecab9c8a894f)) - [@mroetsc](https://github.com/mroetsc)

- - -

## [0.1.2](https://github.com/mroetsc/stuart/compare/1b00b374ad4afc49a0a61f56295724148c857f58..0.1.2) - 2026-06-13
#### Features
- (**cli**) added arg for setting outgoing newline encoding - ([7b3fc5b](https://github.com/mroetsc/stuart/commit/7b3fc5b3736e40d7ca5b046b6f62460b5cac5f79)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) added argument for local echo; also added to settings and state struct - ([f87d6a9](https://github.com/mroetsc/stuart/commit/f87d6a90048d58790d35689ef03bd2402c2c5438)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) added option to not lock the port - ([1b00b37](https://github.com/mroetsc/stuart/commit/1b00b374ad4afc49a0a61f56295724148c857f58)) - [@mroetsc](https://github.com/mroetsc)
- (**serial**) enum for newline encodings - ([bab71f2](https://github.com/mroetsc/stuart/commit/bab71f2c64ca080660e0362d0ffb1ede03e0910c)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) echo_local function to show locally typed bytes - ([96fa0c8](https://github.com/mroetsc/stuart/commit/96fa0c8dd3caabc928184c861184104bd67ba013)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) added outgoing newline option in settings - ([5c11339](https://github.com/mroetsc/stuart/commit/5c1133978ba061762bfee0e5d7368ef8ee3785ff)) - [@mroetsc](https://github.com/mroetsc)
- send newline configured newline to connected device - ([42ce5bb](https://github.com/mroetsc/stuart/commit/42ce5bb98197a5d227a105767a5c6ed99089bca3)) - [@mroetsc](https://github.com/mroetsc)
#### Bug Fixes
- (**ui**) control mode hardly readable on light terminals - ([d0c79b2](https://github.com/mroetsc/stuart/commit/d0c79b2e4b272040bbca0a510ae89b80aa5ed6d9)) - [@mroetsc](https://github.com/mroetsc)
- no lock can only work on unix systems - ([f474100](https://github.com/mroetsc/stuart/commit/f4741009a7cd5b50ea4cb7d9259b87cc3487bc60)) - [@mroetsc](https://github.com/mroetsc)
#### Documentation
- updated readme with new features - ([2ac0bc8](https://github.com/mroetsc/stuart/commit/2ac0bc82e6205c68042f781061f02a8d48ec9d94)) - [@mroetsc](https://github.com/mroetsc)
#### Refactoring
- (**ui**) settings page now uses a common enum; also overhauled styling a bit - ([9a70bff](https://github.com/mroetsc/stuart/commit/9a70bff91790873b2ab2f250b7f95bcc722ea8c1)) - [@mroetsc](https://github.com/mroetsc)
#### Miscellaneous Chores
- (**docs**) reduced demo gif size - ([9fd2f15](https://github.com/mroetsc/stuart/commit/9fd2f158df12504a5944d5c8ee7027d5c3862f1d)) - [@mroetsc](https://github.com/mroetsc)

- - -

## [0.1.1](https://github.com/mroetsc/stuart/compare/a6d2e500aca731bc424bfd7796c3e65f45123fe6..0.1.1) - 2026-06-11
#### Features
- (**cli**) better grouping for help output - ([7a77410](https://github.com/mroetsc/stuart/commit/7a77410cbb52cc0be74861b74bb6a9956f59596a)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) sort available ports in PortSelection properly - ([74f9d11](https://github.com/mroetsc/stuart/commit/74f9d11e5534f5ea6c96f01379eba380b92f6397)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) fallback keycode to enter control mode on emulators not supporting kitty keyboard protocol - ([b179f0f](https://github.com/mroetsc/stuart/commit/b179f0f61dd4ca5548bf39850b55256222261242)) - [@mroetsc](https://github.com/mroetsc)
#### Documentation
- updated readme with with latest new help output and keybindings note - ([a4bf4d2](https://github.com/mroetsc/stuart/commit/a4bf4d20505417eab91f36d17677c3c0dc4c34e8)) - [@mroetsc](https://github.com/mroetsc)
#### Build system
- renamed cargo binary to stuart - ([578950d](https://github.com/mroetsc/stuart/commit/578950dc65ab8d5b3342899e95787f8ea34a42ce)) - [@mroetsc](https://github.com/mroetsc)
#### Miscellaneous Chores
- update cargo package name and update install instructions - ([a6d2e50](https://github.com/mroetsc/stuart/commit/a6d2e500aca731bc424bfd7796c3e65f45123fe6)) - [@mroetsc](https://github.com/mroetsc)

- - -

## [0.1.0](https://github.com/mroetsc/stuart/compare/5434e100f6ec191a832d330b1b5bd8d696a9c922..0.1.0) - 2026-06-10
#### Features
- (**cli**) hold port connection by default; renamed args - ([fd2d0db](https://github.com/mroetsc/stuart/commit/fd2d0dbbe6bc865c39d68da2d7f0477cd0013e4f)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) extracted cli arg parsing into separate file; included new args for PortConfig - ([17ee15b](https://github.com/mroetsc/stuart/commit/17ee15b9ae85cf8aa71af9ba0f64848f46721489)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) added shell completion generation - ([aa82726](https://github.com/mroetsc/stuart/commit/aa82726efb20b47b9388809cf05024c129afed1f)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) open port directly via arg - ([aac5759](https://github.com/mroetsc/stuart/commit/aac57595df73e5cc644897aaf46b8bef27538dee)) - [@mroetsc](https://github.com/mroetsc)
- (**cli**) argument parsing with clap - ([e4dfcfd](https://github.com/mroetsc/stuart/commit/e4dfcfdf7ad3d67bfbb79e8b99fe714b3aab3df4)) - [@mroetsc](https://github.com/mroetsc)
- (**serial**) support for setting data/stop bits, parity and flow control - ([2ed62eb](https://github.com/mroetsc/stuart/commit/2ed62eb8984e39fdce461d33eac63f97d15a038b)) - [@mroetsc](https://github.com/mroetsc)
- (**serial**) sending and receiving bytes - ([da74413](https://github.com/mroetsc/stuart/commit/da74413435a26fc070d3f0fa547a3a236c788ada)) - [@mroetsc](https://github.com/mroetsc)
- (**serial**) thread handling for serial connection - ([0bea09a](https://github.com/mroetsc/stuart/commit/0bea09ae2da464f3db866c4365b460c42ca8cc8f)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) updated to use new PortConfig struct with more options - ([aaf8d5e](https://github.com/mroetsc/stuart/commit/aaf8d5e07aed73485b2578160aa150185332f9e8)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) support for better error handling - ([68fa6ec](https://github.com/mroetsc/stuart/commit/68fa6ecbd18155ce3bd65dfa1082a1233b9b8206)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) support holding port until reconnection - ([70c8a6c](https://github.com/mroetsc/stuart/commit/70c8a6cc99f2bac9bdb0a31d0706f81688d24b4f)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) support for scrollback buffer - ([d5508ca](https://github.com/mroetsc/stuart/commit/d5508cad12e2dda95911d2fd03f078f19f7b3fad)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) copy latest scrolback into clipboard - ([2f07bcd](https://github.com/mroetsc/stuart/commit/2f07bcde9a843a5b99ea457fc09b3d9116356c52)) - [@mroetsc](https://github.com/mroetsc)
- (**state**) new handling functions - ([2a62a6c](https://github.com/mroetsc/stuart/commit/2a62a6c3301b7d557879cc544f5476e11663ccf8)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) implemented basic settings page - ([67d373b](https://github.com/mroetsc/stuart/commit/67d373bc8cfa5f481b78ffb528127fe21305508e)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) separated ui into its own module; also prepared settings dialog - ([606860d](https://github.com/mroetsc/stuart/commit/606860d9fcefb951fa06bd08ac1d2877907904e8)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display errors in title bar - ([9893908](https://github.com/mroetsc/stuart/commit/98939084febc164db29106025a572db38c45f814)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) move reconnecting and scrollback info to top right again - ([89f5195](https://github.com/mroetsc/stuart/commit/89f51955cfba59d27db45ea0fdcc814b797721fa)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) wrap lines in help and info bar when screen is too small - ([ed24375](https://github.com/mroetsc/stuart/commit/ed243754516b40eb8df6d74d4b8ae3b5ca3e89f4)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) better styling for all components - ([f1cf403](https://github.com/mroetsc/stuart/commit/f1cf4038dbcf399f4053af49bcc988b580c1eb56)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) return to port selection screen - ([3a1ec24](https://github.com/mroetsc/stuart/commit/3a1ec2495a2dc329d0101cfac455b6d9fd12a08f)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) refresh button in port selection screen - ([736bc4b](https://github.com/mroetsc/stuart/commit/736bc4b37c5540965799a65e807d571c011001ae)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display reconnecting state - ([daac175](https://github.com/mroetsc/stuart/commit/daac175cab6a38d7dfdf8bf91e88e604afdbdcf9)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) extracted info area into its own panel - ([79607b2](https://github.com/mroetsc/stuart/commit/79607b213b9fc66a838ec69099673909eb79ebe8)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) navigate scrollback - ([eabf0e8](https://github.com/mroetsc/stuart/commit/eabf0e860a76cadb3cd6f608017b56a2a0386a88)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) baud rate change in control mode - ([325e118](https://github.com/mroetsc/stuart/commit/325e118b337645e81df58e4c62e0a70a03592ffa)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) clear screen action in control mode - ([c8f4879](https://github.com/mroetsc/stuart/commit/c8f487937ffc2456f49c569becf4ef2918bf48f8)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) replaced manual keycode matching with terminput - ([97aa956](https://github.com/mroetsc/stuart/commit/97aa95606a2992df3e424e9bf088e5cd934acd21)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display current baud - ([9a3b3cd](https://github.com/mroetsc/stuart/commit/9a3b3cdc9fd1708003a42ea9e611390a6a628c73)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) improved event loop - ([722ac39](https://github.com/mroetsc/stuart/commit/722ac397a9a7615d8d70751ad8538dab1be9d9f0)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) capure and send SIGINT, EOF and SIGTSTP - ([1ebe86e](https://github.com/mroetsc/stuart/commit/1ebe86e52db45e6c39205f836c1cbca88bf98d49)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) use vt100 for terminal rendering - ([2bd3e45](https://github.com/mroetsc/stuart/commit/2bd3e4584ff91e15cf96689ada080ff3dc7d6574)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display cursor - ([866d4ec](https://github.com/mroetsc/stuart/commit/866d4ec96bffd3497930ec148139bb2816ec8e2d)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display current active port - ([6a48020](https://github.com/mroetsc/stuart/commit/6a4802011980cf275054e5b93cea3d6ced90a196)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) Control and Insert mode - ([31e27dc](https://github.com/mroetsc/stuart/commit/31e27dc750cc6ebe9c5758b6a2bd1beda02d07a5)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) draw terminal and use new state functions - ([d014d45](https://github.com/mroetsc/stuart/commit/d014d451f2c670618441ba16ca0a6196ae0f9d67)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) display SerialPortType along with usb information in port selection screen - ([4be95d5](https://github.com/mroetsc/stuart/commit/4be95d57633568924fbd58614528def4436deabe)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) basic ui with port basic serial port selection - ([beec190](https://github.com/mroetsc/stuart/commit/beec19035720c8296457d420363ec16e0eb9c545)) - [@mroetsc](https://github.com/mroetsc)
#### Bug Fixes
- (**state**) baud rate change needs time release the port - ([9a87683](https://github.com/mroetsc/stuart/commit/9a8768320c5242b904fb74064616ff1191696d25)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) handle clipboard paste via bracketed paste mode - ([acccd6d](https://github.com/mroetsc/stuart/commit/acccd6d66291e02db3430dc68565292262e72560)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) enter doing two new lines - ([ad70dc2](https://github.com/mroetsc/stuart/commit/ad70dc251d175c4c5c9de25b46bee2c3b78e883a)) - [@mroetsc](https://github.com/mroetsc)
- (**ui**) scroll with buffer - ([0c353ce](https://github.com/mroetsc/stuart/commit/0c353ceca08966c4211e6a15c02832d3493f8a93)) - [@mroetsc](https://github.com/mroetsc)
#### Documentation
- updated readme with more detailed information and demo - ([a3ed6bb](https://github.com/mroetsc/stuart/commit/a3ed6bb279cb0e8212c5651dc4fac084d8f4f460)) - [@mroetsc](https://github.com/mroetsc)
- added demo gif - ([c778eec](https://github.com/mroetsc/stuart/commit/c778eecfca5ef777a73984badb6eda665c1821be)) - [@mroetsc](https://github.com/mroetsc)
#### Build system
- created cocogitto config - ([d2cbb36](https://github.com/mroetsc/stuart/commit/d2cbb36d7a40c041c28cb9126a764aea22468505)) - [@mroetsc](https://github.com/mroetsc)
#### Continuous Integration
- fix issues found in testing - ([22159d1](https://github.com/mroetsc/stuart/commit/22159d1d8d402f4f2a8d00d72caee771872bd237)) - [@mroetsc](https://github.com/mroetsc)
- build and release pipeline - ([19e8509](https://github.com/mroetsc/stuart/commit/19e8509692e1b0c61bd5f6794c4a45dd2071dc11)) - [@mroetsc](https://github.com/mroetsc)
#### Refactoring
- (**ui**) renamed clear to flush to have c key available for copy - ([42fbbdf](https://github.com/mroetsc/stuart/commit/42fbbdf8159f7b281b228ec5d6dd80c2b43f4277)) - [@mroetsc](https://github.com/mroetsc)
- moved application state into its own module - ([e4e8a1a](https://github.com/mroetsc/stuart/commit/e4e8a1a7db57e4e76c4e282f9580142b2177be94)) - [@mroetsc](https://github.com/mroetsc)
#### Miscellaneous Chores
- (**ui**) display baud from port config - ([fd6113d](https://github.com/mroetsc/stuart/commit/fd6113d9d33b3e66563ff8891c9cdc4186878aeb)) - [@mroetsc](https://github.com/mroetsc)
- added strip-ansi-escapes dependency - ([2c193df](https://github.com/mroetsc/stuart/commit/2c193dfe6fd49cc7098a43b06aebb1f737f1ce3f)) - [@mroetsc](https://github.com/mroetsc)
- added clap_complete dependency - ([9344a9a](https://github.com/mroetsc/stuart/commit/9344a9ac0187058b88f7f9ee7d7eacc4bc82fd25)) - [@mroetsc](https://github.com/mroetsc)
- added arboard dependency - ([54434ab](https://github.com/mroetsc/stuart/commit/54434abf319382698d992122a0e0aaeadd2f25b9)) - [@mroetsc](https://github.com/mroetsc)
- added terminput dependency - ([726556e](https://github.com/mroetsc/stuart/commit/726556ec835a145083ec02158ce8e245a8774e30)) - [@mroetsc](https://github.com/mroetsc)
- added vt100 dependency - ([2a20e8d](https://github.com/mroetsc/stuart/commit/2a20e8d18cd65320435f24c93f57cad83c5fef00)) - [@mroetsc](https://github.com/mroetsc)
- applied clippy suggestion - ([55f544e](https://github.com/mroetsc/stuart/commit/55f544e8c7d7c7c3a901fa906a72da83f0e537ba)) - [@mroetsc](https://github.com/mroetsc)
- added serialport dependency - ([9d6a14d](https://github.com/mroetsc/stuart/commit/9d6a14dd0c7ba3cb127944fcaea5539440040c1d)) - [@mroetsc](https://github.com/mroetsc)
- added dependencies - ([04c4677](https://github.com/mroetsc/stuart/commit/04c46778ee195f461d96462bb67eabee95ba9ca8)) - [@mroetsc](https://github.com/mroetsc)
- initial commit - ([5434e10](https://github.com/mroetsc/stuart/commit/5434e100f6ec191a832d330b1b5bd8d696a9c922)) - [@mroetsc](https://github.com/mroetsc)

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).