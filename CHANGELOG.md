# Changelog

## [0.4.2](https://github.com/martinohmann/dts/compare/v0.4.1...v0.4.2) (2022-07-13)


### Miscellaneous

* **deps:** bump clap from 3.2.8 to 3.2.11 ([#96](https://github.com/martinohmann/dts/issues/96)) ([148e54e](https://github.com/martinohmann/dts/commit/148e54e2be3f61a85abfacaa7dc5c1e9920d97a8))
* **deps:** bump criterion from 0.3.5 to 0.3.6 ([#94](https://github.com/martinohmann/dts/issues/94)) ([47adb70](https://github.com/martinohmann/dts/commit/47adb70a29ab5aba91697cca6a430624e53f97d9))
* **deps:** bump hcl-rs from 0.6.1 to 0.6.2 ([#95](https://github.com/martinohmann/dts/issues/95)) ([922797b](https://github.com/martinohmann/dts/commit/922797b370d287dfd60f075d241ea4db76a417c5))
* **deps:** bump once_cell from 1.12.0 to 1.13.0 ([#97](https://github.com/martinohmann/dts/issues/97)) ([e8cd71a](https://github.com/martinohmann/dts/commit/e8cd71ab2dc7428606f9a651c14a7f3eb9eaaf5c))
* **deps:** bump regex from 1.5.6 to 1.6.0 ([#99](https://github.com/martinohmann/dts/issues/99)) ([774c602](https://github.com/martinohmann/dts/commit/774c6029d9ec82753b374f90721db84ba11824cf))
* **deps:** bump serde from 1.0.137 to 1.0.139 ([#93](https://github.com/martinohmann/dts/issues/93)) ([1e9069c](https://github.com/martinohmann/dts/commit/1e9069c091076dfe2222c2d5198639bf8d2071f4))
* **deps:** bump serde_qs from 0.9.2 to 0.10.1 ([#102](https://github.com/martinohmann/dts/issues/102)) ([ceaada2](https://github.com/martinohmann/dts/commit/ceaada2f5b4b2bda5c7dd7bbc577338a199d60dd))
* **deps:** bump serde_yaml from 0.8.24 to 0.8.25 ([#100](https://github.com/martinohmann/dts/issues/100)) ([0bb90c8](https://github.com/martinohmann/dts/commit/0bb90c822734339855d42f8c50834dacdc2514f5))
* **deps:** bump ureq from 2.4.0 to 2.5.0 ([#101](https://github.com/martinohmann/dts/issues/101)) ([54847b2](https://github.com/martinohmann/dts/commit/54847b2cf738cbfe7eea3a5d609e32e08d1d5dd3))

## [0.4.1](https://github.com/martinohmann/dts/compare/v0.4.0...v0.4.1) (2022-07-01)


### Miscellaneous

* **deps:** bump anyhow from 1.0.57 to 1.0.58 ([#89](https://github.com/martinohmann/dts/issues/89)) ([8ff8106](https://github.com/martinohmann/dts/commit/8ff8106e87bd554c27a2cb25d444bd7ecff4ec3c))
* **deps:** bump clap from 3.2.5 to 3.2.8 ([#91](https://github.com/martinohmann/dts/issues/91)) ([fbf73c8](https://github.com/martinohmann/dts/commit/fbf73c87483525d7cd194056ea1e219f94fc8039))
* **deps:** bump clap_complete from 3.2.1 to 3.2.3 ([#88](https://github.com/martinohmann/dts/issues/88)) ([8fd14b6](https://github.com/martinohmann/dts/commit/8fd14b6391e18b9216b55f6866bd2c66d23cb466))
* **deps:** bump crossbeam-utils from 0.8.9 to 0.8.10 ([#90](https://github.com/martinohmann/dts/issues/90)) ([8d387e4](https://github.com/martinohmann/dts/commit/8d387e4c080caf23f72e81d46d283c741c0b1e96))
* **deps:** bump serde_json from 1.0.81 to 1.0.82 ([#87](https://github.com/martinohmann/dts/issues/87)) ([7fa2cda](https://github.com/martinohmann/dts/commit/7fa2cda147c0535184b43eeca0b7a6bd310c906e))

## [0.4.0](https://github.com/martinohmann/dts/compare/v0.3.1...v0.4.0) (2022-06-26)


### Features

* add `jaq` feature ([#84](https://github.com/martinohmann/dts/issues/84)) ([6525db7](https://github.com/martinohmann/dts/commit/6525db75037ad48b09e9a5025996cb05fece9166))
* serialization support for HCL ([#85](https://github.com/martinohmann/dts/issues/85)) ([a90e23b](https://github.com/martinohmann/dts/commit/a90e23b0e03de08d549b0d8c5fbf0abfc859703d))


### Bug Fixes

* unexport filter modules ([6a59d64](https://github.com/martinohmann/dts/commit/6a59d64bfa6a2b6cd24e302fdde3740d31d55891))

## [0.3.1](https://github.com/martinohmann/dts/compare/v0.3.0...v0.3.1) (2022-06-17)


### Bug Fixes

* **security:** bump crossbeam-utils to 0.8.9 ([f6fcaa9](https://github.com/martinohmann/dts/commit/f6fcaa92b62f1f3e72a4fc78bb18d63b0451eaa1))


### Miscellaneous

* **deps:** upgrade clap to 3.2.5 ([82eb321](https://github.com/martinohmann/dts/commit/82eb321fe69f96997a522cb82f11b7612a548459))

## [0.3.0](https://github.com/martinohmann/dts/compare/v0.2.0...v0.3.0) (2022-06-17)


### ⚠ BREAKING CHANGES

* unify crates `dts` and `dts-core` (#81)

### Bug Fixes

* disable termcap initialization for `less` ([#55](https://github.com/martinohmann/dts/issues/55)) ([07c002f](https://github.com/martinohmann/dts/commit/07c002f554711f150d5060ee732b5cc4bd99b2ac)), closes [#54](https://github.com/martinohmann/dts/issues/54)
* do not remove empty segments from deserialized text ([37af5b0](https://github.com/martinohmann/dts/commit/37af5b0ce8e07396d17ff09d63daf6d35556066c))
* optimize release build for binary size ([#53](https://github.com/martinohmann/dts/issues/53)) ([36eefb8](https://github.com/martinohmann/dts/commit/36eefb83ecf2ee920f6262316f7acfc9b681d710))
* publish artifacts on any tag ([affa8d4](https://github.com/martinohmann/dts/commit/affa8d4dfe87f9a37ec5d1439fef6b3c404426f7))
* remove `strip` feature from release profile ([#58](https://github.com/martinohmann/dts/issues/58)) ([b9c04b8](https://github.com/martinohmann/dts/commit/b9c04b80698e38363df3539ed89b1669a09803a8))
* remove custom jq error type ([a392342](https://github.com/martinohmann/dts/commit/a392342eaedd8171a692a610fd0234a30319356a))
* unify crates `dts` and `dts-core` ([#81](https://github.com/martinohmann/dts/issues/81)) ([24e203e](https://github.com/martinohmann/dts/commit/24e203e8f361c1e0489e476b232d9bb75f3ee469))
* update `Cargo.lock` ([0575496](https://github.com/martinohmann/dts/commit/0575496b3bf3b42d22890a9ec1165ff9840fe675))


### Miscellaneous

* add .release-please-manifest.json ([401b9d3](https://github.com/martinohmann/dts/commit/401b9d3f4828089a45a226ec37a28711e8901dfb))
* add release-please-config.json ([e437bc8](https://github.com/martinohmann/dts/commit/e437bc830e121f7c10f90ebf32f9dd903563fe4d))
* **deps:** bump actions/cache from 2 to 3 ([#61](https://github.com/martinohmann/dts/issues/61)) ([cfc48e7](https://github.com/martinohmann/dts/commit/cfc48e7ace8edcb7b14019964fda84cc698311b3))
* **deps:** bump actions/checkout from 2 to 3 ([#36](https://github.com/martinohmann/dts/issues/36)) ([382bc4c](https://github.com/martinohmann/dts/commit/382bc4cccc617c62e8bb773b113d8c02eff6b946))
* **deps:** bump anyhow from 1.0.51 to 1.0.56 ([#51](https://github.com/martinohmann/dts/issues/51)) ([f65cada](https://github.com/martinohmann/dts/commit/f65cadaea21e3db96a59720e07ad78afb1c5a7bc))
* **deps:** bump anyhow from 1.0.56 to 1.0.57 ([#68](https://github.com/martinohmann/dts/issues/68)) ([cca04ad](https://github.com/martinohmann/dts/commit/cca04ad9e06571f649a827d3fdfc44bf0e801856))
* **deps:** bump assert_cmd from 2.0.2 to 2.0.4 ([#50](https://github.com/martinohmann/dts/issues/50)) ([edebcde](https://github.com/martinohmann/dts/commit/edebcdee1d23697f3308ff3c45e14b931430b271))
* **deps:** bump bat from 0.18.3 to 0.20.0 ([#48](https://github.com/martinohmann/dts/issues/48)) ([bfb4c88](https://github.com/martinohmann/dts/commit/bfb4c88e92cb310d5c37e32e12d7be62cee0d6e5))
* **deps:** bump bat from 0.20.0 to 0.21.0 ([#75](https://github.com/martinohmann/dts/issues/75)) ([5dad3fb](https://github.com/martinohmann/dts/commit/5dad3fbad170ce83f88d10ac167a88dc28bba02a))
* **deps:** bump hcl-rs from 0.2.0 to 0.2.1 ([#60](https://github.com/martinohmann/dts/issues/60)) ([e84d9e1](https://github.com/martinohmann/dts/commit/e84d9e1aba497a2aa9d3501813bde81267953027))
* **deps:** bump hcl-rs from 0.2.1 to 0.3.3 ([#62](https://github.com/martinohmann/dts/issues/62)) ([9d9758c](https://github.com/martinohmann/dts/commit/9d9758c71ea7488ecc57f822eb60473ff5373821))
* **deps:** bump once_cell from 1.10.0 to 1.12.0 ([#76](https://github.com/martinohmann/dts/issues/76)) ([e11374e](https://github.com/martinohmann/dts/commit/e11374e799f869d2dd95951cf5d04f524e08ea5c))
* **deps:** bump once_cell from 1.9.0 to 1.10.0 ([#49](https://github.com/martinohmann/dts/issues/49)) ([7940fc5](https://github.com/martinohmann/dts/commit/7940fc51815b4a3dda68d5a985433fdd11fd8cda))
* **deps:** bump predicates from 2.1.0 to 2.1.1 ([#44](https://github.com/martinohmann/dts/issues/44)) ([c5c7c67](https://github.com/martinohmann/dts/commit/c5c7c67b29128ef7de9eff83ab969e6226ac0203))
* **deps:** bump pretty_assertions from 1.0.0 to 1.1.0 ([#40](https://github.com/martinohmann/dts/issues/40)) ([09fa4f7](https://github.com/martinohmann/dts/commit/09fa4f72e81d8a267c02b3bf8f73a9db0c3ae970))
* **deps:** bump pretty_assertions from 1.1.0 to 1.2.0 ([#59](https://github.com/martinohmann/dts/issues/59)) ([9855f34](https://github.com/martinohmann/dts/commit/9855f3446ff431feb4d34a85067747fcb3b5a06c))
* **deps:** bump pretty_assertions from 1.2.0 to 1.2.1 ([#65](https://github.com/martinohmann/dts/issues/65)) ([d445cef](https://github.com/martinohmann/dts/commit/d445cef0206407d66725b549ac4fd62cf5c72ef8))
* **deps:** bump rayon from 1.5.1 to 1.5.2 ([#66](https://github.com/martinohmann/dts/issues/66)) ([2bf7485](https://github.com/martinohmann/dts/commit/2bf7485bd813331746d681fcee4818945fb6a164))
* **deps:** bump rayon from 1.5.2 to 1.5.3 ([#77](https://github.com/martinohmann/dts/issues/77)) ([9389f3f](https://github.com/martinohmann/dts/commit/9389f3fe89c9a298e41341e64019af8a1896cdb6))
* **deps:** bump regex from 1.5.4 to 1.5.5 ([#46](https://github.com/martinohmann/dts/issues/46)) ([28b9ed2](https://github.com/martinohmann/dts/commit/28b9ed28eb91b2da0c93b0016f877dfa8a2a4780))
* **deps:** bump regex from 1.5.5 to 1.5.6 ([#78](https://github.com/martinohmann/dts/issues/78)) ([13f7ae5](https://github.com/martinohmann/dts/commit/13f7ae5f36a1456562721d7bb4dd7aafaaf322cf))
* **deps:** bump serde from 1.0.132 to 1.0.136 ([#37](https://github.com/martinohmann/dts/issues/37)) ([d16fb4c](https://github.com/martinohmann/dts/commit/d16fb4cb732c75f844b9eb1ea1bbd8ec40143470))
* **deps:** bump serde from 1.0.136 to 1.0.137 ([#70](https://github.com/martinohmann/dts/issues/70)) ([a3b424c](https://github.com/martinohmann/dts/commit/a3b424ca6da96aa4287758870cc77047d7f37c22))
* **deps:** bump serde_json from 1.0.73 to 1.0.79 ([#43](https://github.com/martinohmann/dts/issues/43)) ([c0b4487](https://github.com/martinohmann/dts/commit/c0b4487e721e86bfd52f30071d9819e4fe56ff0a))
* **deps:** bump serde_json from 1.0.79 to 1.0.80 ([#67](https://github.com/martinohmann/dts/issues/67)) ([6a867ac](https://github.com/martinohmann/dts/commit/6a867acedead329309d0a1768fd4275bda7c451a))
* **deps:** bump serde_qs from 0.8.5 to 0.9.1 ([#39](https://github.com/martinohmann/dts/issues/39)) ([ef87acc](https://github.com/martinohmann/dts/commit/ef87accc470777f400d7d13a79af118720987232))
* **deps:** bump serde_qs from 0.9.1 to 0.9.2 ([#74](https://github.com/martinohmann/dts/issues/74)) ([752bbc6](https://github.com/martinohmann/dts/commit/752bbc6aa171c8520ce23dfbc72e9e08e8b90433))
* **deps:** bump serde_yaml from 0.8.23 to 0.8.24 ([#72](https://github.com/martinohmann/dts/issues/72)) ([0f8f63e](https://github.com/martinohmann/dts/commit/0f8f63e47021c2817df50cb59c59bc7d97d2e182))
* **deps:** bump shell-words from 1.0.0 to 1.1.0 ([#42](https://github.com/martinohmann/dts/issues/42)) ([0d542cc](https://github.com/martinohmann/dts/commit/0d542cc64e5893aac5900d49b65e576d9f51c4ff))
* **deps:** bump termcolor from 1.1.2 to 1.1.3 ([#47](https://github.com/martinohmann/dts/issues/47)) ([3f2f484](https://github.com/martinohmann/dts/commit/3f2f484a50236558b5416cbd040530b8092126be))
* **deps:** bump thiserror from 1.0.30 to 1.0.31 ([#69](https://github.com/martinohmann/dts/issues/69)) ([9839ed6](https://github.com/martinohmann/dts/commit/9839ed6636b01e25f34c77c2434b5bf2de45613d))
* **deps:** bump toml from 0.5.8 to 0.5.9 ([#73](https://github.com/martinohmann/dts/issues/73)) ([3b027f4](https://github.com/martinohmann/dts/commit/3b027f4ac4f1cdf3d4b96f8c249954b85474be4b))
* release main ([#30](https://github.com/martinohmann/dts/issues/30)) ([1d51227](https://github.com/martinohmann/dts/commit/1d51227ab8b69ef4602d143f81766d30212082de))
* release main ([#32](https://github.com/martinohmann/dts/issues/32)) ([f617297](https://github.com/martinohmann/dts/commit/f617297717c4d8d0187ea447c7bdd28c26e2a5be))
* release main ([#45](https://github.com/martinohmann/dts/issues/45)) ([14059b5](https://github.com/martinohmann/dts/commit/14059b55b37c77fc762a4870b658eccb98492676))
* release main ([#56](https://github.com/martinohmann/dts/issues/56)) ([f9c7acc](https://github.com/martinohmann/dts/commit/f9c7acc0ba59a595794006d790b8f0dee11449b9))
* release main ([#57](https://github.com/martinohmann/dts/issues/57)) ([569303d](https://github.com/martinohmann/dts/commit/569303da7b648a8c7025ed52f2296450e7235943))
* release main ([#63](https://github.com/martinohmann/dts/issues/63)) ([a8c3d47](https://github.com/martinohmann/dts/commit/a8c3d47050dd4b05d0a5d1ee6eaae1013446a1ec))
* release main ([#64](https://github.com/martinohmann/dts/issues/64)) ([34299fe](https://github.com/martinohmann/dts/commit/34299fe27bff8488b5ab1800196be8095aa09c8e))
* remove dts-core changelog ([a492e7b](https://github.com/martinohmann/dts/commit/a492e7b05f27363bb719d50a37b2f51d7d47fa8f))
* remove redundant release-type ([9fe3536](https://github.com/martinohmann/dts/commit/9fe3536feba453b359533e305c515d0643bf421d))

## [0.2.7](https://github.com/martinohmann/dts/compare/dts-v0.2.6...dts-v0.2.7) (2022-06-03)


### Bug Fixes

* remove custom jq error type ([a392342](https://github.com/martinohmann/dts/commit/a392342eaedd8171a692a610fd0234a30319356a))


### Miscellaneous

* **deps:** bump anyhow from 1.0.56 to 1.0.57 ([#68](https://github.com/martinohmann/dts/issues/68)) ([cca04ad](https://github.com/martinohmann/dts/commit/cca04ad9e06571f649a827d3fdfc44bf0e801856))
* **deps:** bump bat from 0.20.0 to 0.21.0 ([#75](https://github.com/martinohmann/dts/issues/75)) ([5dad3fb](https://github.com/martinohmann/dts/commit/5dad3fbad170ce83f88d10ac167a88dc28bba02a))
* **deps:** bump once_cell from 1.10.0 to 1.12.0 ([#76](https://github.com/martinohmann/dts/issues/76)) ([e11374e](https://github.com/martinohmann/dts/commit/e11374e799f869d2dd95951cf5d04f524e08ea5c))
* **deps:** bump pretty_assertions from 1.2.0 to 1.2.1 ([#65](https://github.com/martinohmann/dts/issues/65)) ([d445cef](https://github.com/martinohmann/dts/commit/d445cef0206407d66725b549ac4fd62cf5c72ef8))
* **deps:** bump rayon from 1.5.1 to 1.5.2 ([#66](https://github.com/martinohmann/dts/issues/66)) ([2bf7485](https://github.com/martinohmann/dts/commit/2bf7485bd813331746d681fcee4818945fb6a164))
* **deps:** bump rayon from 1.5.2 to 1.5.3 ([#77](https://github.com/martinohmann/dts/issues/77)) ([9389f3f](https://github.com/martinohmann/dts/commit/9389f3fe89c9a298e41341e64019af8a1896cdb6))
* **deps:** bump regex from 1.5.5 to 1.5.6 ([#78](https://github.com/martinohmann/dts/issues/78)) ([13f7ae5](https://github.com/martinohmann/dts/commit/13f7ae5f36a1456562721d7bb4dd7aafaaf322cf))
* **deps:** bump serde from 1.0.136 to 1.0.137 ([#70](https://github.com/martinohmann/dts/issues/70)) ([a3b424c](https://github.com/martinohmann/dts/commit/a3b424ca6da96aa4287758870cc77047d7f37c22))
* **deps:** bump serde_json from 1.0.79 to 1.0.80 ([#67](https://github.com/martinohmann/dts/issues/67)) ([6a867ac](https://github.com/martinohmann/dts/commit/6a867acedead329309d0a1768fd4275bda7c451a))
* **deps:** bump serde_qs from 0.9.1 to 0.9.2 ([#74](https://github.com/martinohmann/dts/issues/74)) ([752bbc6](https://github.com/martinohmann/dts/commit/752bbc6aa171c8520ce23dfbc72e9e08e8b90433))
* **deps:** bump serde_yaml from 0.8.23 to 0.8.24 ([#72](https://github.com/martinohmann/dts/issues/72)) ([0f8f63e](https://github.com/martinohmann/dts/commit/0f8f63e47021c2817df50cb59c59bc7d97d2e182))
* **deps:** bump thiserror from 1.0.30 to 1.0.31 ([#69](https://github.com/martinohmann/dts/issues/69)) ([9839ed6](https://github.com/martinohmann/dts/commit/9839ed6636b01e25f34c77c2434b5bf2de45613d))
* **deps:** bump toml from 0.5.8 to 0.5.9 ([#73](https://github.com/martinohmann/dts/issues/73)) ([3b027f4](https://github.com/martinohmann/dts/commit/3b027f4ac4f1cdf3d4b96f8c249954b85474be4b))

### [0.2.6](https://github.com/martinohmann/dts/compare/dts-v0.2.5...dts-v0.2.6) (2022-03-27)


### Miscellaneous

* **deps:** bump hcl-rs from 0.2.1 to 0.3.3 ([#62](https://github.com/martinohmann/dts/issues/62)) ([9d9758c](https://github.com/martinohmann/dts/commit/9d9758c71ea7488ecc57f822eb60473ff5373821))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.5 to 0.2.6

### [0.2.5](https://github.com/martinohmann/dts/compare/dts-v0.2.4...dts-v0.2.5) (2022-03-23)


### Bug Fixes

* remove `strip` feature from release profile ([#58](https://github.com/martinohmann/dts/issues/58)) ([b9c04b8](https://github.com/martinohmann/dts/commit/b9c04b80698e38363df3539ed89b1669a09803a8))


### Miscellaneous

* **deps:** bump actions/cache from 2 to 3 ([#61](https://github.com/martinohmann/dts/issues/61)) ([cfc48e7](https://github.com/martinohmann/dts/commit/cfc48e7ace8edcb7b14019964fda84cc698311b3))
* **deps:** bump bat from 0.18.3 to 0.20.0 ([#48](https://github.com/martinohmann/dts/issues/48)) ([bfb4c88](https://github.com/martinohmann/dts/commit/bfb4c88e92cb310d5c37e32e12d7be62cee0d6e5))
* **deps:** bump hcl-rs from 0.2.0 to 0.2.1 ([#60](https://github.com/martinohmann/dts/issues/60)) ([e84d9e1](https://github.com/martinohmann/dts/commit/e84d9e1aba497a2aa9d3501813bde81267953027))
* **deps:** bump pretty_assertions from 1.1.0 to 1.2.0 ([#59](https://github.com/martinohmann/dts/issues/59)) ([9855f34](https://github.com/martinohmann/dts/commit/9855f3446ff431feb4d34a85067747fcb3b5a06c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.4 to 0.2.5

### [0.2.4](https://github.com/martinohmann/dts/compare/dts-v0.2.3...dts-v0.2.4) (2022-03-16)


### Bug Fixes

* disable termcap initialization for `less` ([#55](https://github.com/martinohmann/dts/issues/55)) ([07c002f](https://github.com/martinohmann/dts/commit/07c002f554711f150d5060ee732b5cc4bd99b2ac)), closes [#54](https://github.com/martinohmann/dts/issues/54)
* optimize release build for binary size ([#53](https://github.com/martinohmann/dts/issues/53)) ([36eefb8](https://github.com/martinohmann/dts/commit/36eefb83ecf2ee920f6262316f7acfc9b681d710))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.3 to 0.2.4

### [0.2.3](https://github.com/martinohmann/dts/compare/dts-v0.2.2...dts-v0.2.3) (2022-03-10)


### Miscellaneous

* **deps:** bump actions/checkout from 2 to 3 ([#36](https://github.com/martinohmann/dts/issues/36)) ([382bc4c](https://github.com/martinohmann/dts/commit/382bc4cccc617c62e8bb773b113d8c02eff6b946))
* **deps:** bump anyhow from 1.0.51 to 1.0.56 ([#51](https://github.com/martinohmann/dts/issues/51)) ([f65cada](https://github.com/martinohmann/dts/commit/f65cadaea21e3db96a59720e07ad78afb1c5a7bc))
* **deps:** bump assert_cmd from 2.0.2 to 2.0.4 ([#50](https://github.com/martinohmann/dts/issues/50)) ([edebcde](https://github.com/martinohmann/dts/commit/edebcdee1d23697f3308ff3c45e14b931430b271))
* **deps:** bump once_cell from 1.9.0 to 1.10.0 ([#49](https://github.com/martinohmann/dts/issues/49)) ([7940fc5](https://github.com/martinohmann/dts/commit/7940fc51815b4a3dda68d5a985433fdd11fd8cda))
* **deps:** bump predicates from 2.1.0 to 2.1.1 ([#44](https://github.com/martinohmann/dts/issues/44)) ([c5c7c67](https://github.com/martinohmann/dts/commit/c5c7c67b29128ef7de9eff83ab969e6226ac0203))
* **deps:** bump pretty_assertions from 1.0.0 to 1.1.0 ([#40](https://github.com/martinohmann/dts/issues/40)) ([09fa4f7](https://github.com/martinohmann/dts/commit/09fa4f72e81d8a267c02b3bf8f73a9db0c3ae970))
* **deps:** bump regex from 1.5.4 to 1.5.5 ([#46](https://github.com/martinohmann/dts/issues/46)) ([28b9ed2](https://github.com/martinohmann/dts/commit/28b9ed28eb91b2da0c93b0016f877dfa8a2a4780))
* **deps:** bump serde from 1.0.132 to 1.0.136 ([#37](https://github.com/martinohmann/dts/issues/37)) ([d16fb4c](https://github.com/martinohmann/dts/commit/d16fb4cb732c75f844b9eb1ea1bbd8ec40143470))
* **deps:** bump serde_json from 1.0.73 to 1.0.79 ([#43](https://github.com/martinohmann/dts/issues/43)) ([c0b4487](https://github.com/martinohmann/dts/commit/c0b4487e721e86bfd52f30071d9819e4fe56ff0a))
* **deps:** bump serde_qs from 0.8.5 to 0.9.1 ([#39](https://github.com/martinohmann/dts/issues/39)) ([ef87acc](https://github.com/martinohmann/dts/commit/ef87accc470777f400d7d13a79af118720987232))
* **deps:** bump shell-words from 1.0.0 to 1.1.0 ([#42](https://github.com/martinohmann/dts/issues/42)) ([0d542cc](https://github.com/martinohmann/dts/commit/0d542cc64e5893aac5900d49b65e576d9f51c4ff))
* **deps:** bump termcolor from 1.1.2 to 1.1.3 ([#47](https://github.com/martinohmann/dts/issues/47)) ([3f2f484](https://github.com/martinohmann/dts/commit/3f2f484a50236558b5416cbd040530b8092126be))
* remove dts-core changelog ([a492e7b](https://github.com/martinohmann/dts/commit/a492e7b05f27363bb719d50a37b2f51d7d47fa8f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.2 to 0.2.3

### [0.2.2](https://github.com/martinohmann/dts/compare/dts-v0.2.1...dts-v0.2.2) (2022-03-05)


### Bug Fixes

* publish artifacts on any tag ([affa8d4](https://github.com/martinohmann/dts/commit/affa8d4dfe87f9a37ec5d1439fef6b3c404426f7))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.1 to 0.2.2

### [0.2.1](https://github.com/martinohmann/dts/compare/dts-v0.2.0...dts-v0.2.1) (2022-03-05)


### Bug Fixes

* do not remove empty segments from deserialized text ([37af5b0](https://github.com/martinohmann/dts/commit/37af5b0ce8e07396d17ff09d63daf6d35556066c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * dts-core bumped from 0.2.0 to 0.2.1
