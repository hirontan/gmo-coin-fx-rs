# gmo-coin-fx-rs

## batonel

```shell
curl -fsSL https://raw.githubusercontent.com/Arcflect/batonel/main/scripts/install-batonel.sh | bash -s -- v1.13.0
```

### 初期化

```shell
batonel init --preset rust-clean-hexagonal --project-name gmo-coin-fx-rs --dry-run
batonel init --preset rust-clean-hexagonal --project-name gmo-coin-fx-rs
```

※ `project.baton.yaml`, `placement.rules.yaml`, `contracts.template.yaml`, `artifacts.plan.yaml`, `policy.profile.yaml`, `guard.sidecar.yaml` が生成されるので、適宜修正する

### プラン確認

```shell
batonel plan
```


