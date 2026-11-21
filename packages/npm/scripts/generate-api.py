"""Generate the native dispatch and language bindings from the shared Rust API."""
import argparse
import re
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[1]
core = root.parent / "core" / "src" / "api"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--check", action="store_true", help="Fail if any generated file is stale")
check = parser.parse_args().check
stale = []


def emit(path: Path, content: str) -> None:
    if not check:
        path.write_text(content, encoding="utf-8")
    elif not path.is_file() or path.read_text(encoding="utf-8") != content:
        stale.append(path.relative_to(root.parent.parent))


arms, groups, python_groups = [], {}, {}
python_binary_methods = {
    ("materials", "upload"): (
        "file_name: str, content_type: str, source: bytes",
        "self._client._upload_material(file_name, content_type, source)",
    ),
    ("messages", "uploadMedia"): (
        "target: MediaTarget, file_type: int, source: bytes | str, srv_send_msg: bool",
        "self._upload_media(target, file_type, source, srv_send_msg)",
    ),
    ("messages", "sendMedia"): (
        "target: MediaTarget, file_type: int, source: bytes | str, message: Message",
        "self._send_media(target, file_type, source, message)",
    ),
}
python_types = {
    "string": "str",
    "number": "int",
    "boolean": "bool",
    "Message": "Message",
    "MediaTarget": "MediaTarget",
    "Record<string, unknown>": "dict[str, Any]",
}
media_params = ["target: MediaTarget", "file_type: number", "source: UploadSource"]
binary_methods = {
    ("materials", "upload"): (
        ["file_name: string", "content_type: string", "source: UploadSource"], "uploadMaterial"
    ),
    ("messages", "uploadMedia"): ([*media_params, "srv_send_msg: boolean"], "media.uploadMedia"),
    ("messages", "sendMedia"): ([*media_params, "message: Message"], "media.sendMedia"),
}
for path in sorted(core.rglob("*.rs")):
    if path.name == "upload.rs" or path == core / "mod.rs":
        continue
    group = {"channel": "channels", "group": "groups", "guild": "guilds", "user": "users", "interaction": "interactions", "material": "materials", "panel": "panels", "mod": "messages", "media": "messages"}.get(path.stem, path.stem)
    source = path.read_text(encoding="utf-8")
    for match in re.finditer(r"pub async fn (\w+)\s*\((.*?)\)\s*->\s*Result<([^\n{]+)>", source, re.S):
        name, signature, result_type = match.groups()
        result_type = result_type.strip()
        python_result = (
            "None" if result_type == "()" else
            "Any" if result_type == "Value" else
            "list[dict[str, Any]]" if result_type.startswith("Vec<") else
            "dict[str, Any]"
        )
        camel = re.sub(r"_([a-z])", lambda m: m[1].upper(), name)
        if (group, camel) in binary_methods:
            types, _ = binary_methods[group, camel]
            groups.setdefault(group, []).append((camel, types))
            params, invoke = python_binary_methods[group, camel]
            python_groups.setdefault(group, []).append((name, params, invoke, python_result))
            continue
        params = [part.strip().split(":", 1) for part in signature.split(",") if part.strip() and part.strip() != "&self"]
        declarations, arguments, types = [], [], []
        for index, (param, kind) in enumerate(params):
            kind = kind.strip()
            value = ("args.first()" if index == 0 else f"args.get({index})") + ".cloned().unwrap_or(Value::Null)"
            optional = kind.startswith("Option<")
            if kind == "&str" or kind == "Option<&str>":
                typ = "Option<String>" if optional else "String"
                arg = f"{param}.as_deref()" if optional else f"&{param}"
                ts = "string | null" if optional else "string"
            elif kind.startswith("MediaTarget"):
                declarations.append(f'let {param}_value: MediaTargetInput = MediaTargetInput::parse({value})?;')
                declarations.append(f'let {param} = {param}_value.target()?;')
                arguments.append(param)
                types.append(f"{param}: MediaTarget")
                continue
            elif kind == "impl Into<Sendable>":
                typ, arg, ts = "Value", f"sendable({param})?", "Message"
            else:
                typ = kind.removeprefix("&")
                if typ in ("ImageUploadRequest", "ImageUploadCompletion"):
                    typ = "crate::" + typ
                arg = f"&{param}" if kind.startswith("&") else param
                ts = "boolean" if kind == "bool" else "number | null" if optional else "number" if kind in ("u8", "u16", "u32", "u64") else "Record<string, unknown>"
            declarations.append(f"let {param}: {typ} = serde_json::from_value({value})?;")
            arguments.append(arg)
            types.append(f"{param}{'?' if optional else ''}: {ts}")
        arms.append(f'"{group}.{camel}" => {{ ' + " ".join(declarations) + f" let result = api.{group}().{name}({', '.join(arguments)}).await?; Ok(serde_json::to_value(result)?) " + "},")
        if result_type == "()":
            arms[-1] = arms[-1].replace("let result = ", "").replace("Ok(serde_json::to_value(result)?)", "Ok(Value::Null)")
        groups.setdefault(group, []).append((camel, types))
        python_params = []
        for declaration in types:
            parameter, ts_type = declaration.split(": ", 1)
            optional = parameter.endswith("?")
            parameter = parameter.removesuffix("?")
            py_type = python_types[ts_type.removesuffix(" | null")]
            python_params.append(f"{parameter}: {py_type}" + (" | None = None" if optional else ""))
        python_args = "".join(f", {param}" for param, _ in params)
        invoke = f'self._client._call("{group}.{camel}"{python_args})'
        python_groups.setdefault(group, []).append(
            (name, ", ".join(python_params), invoke, python_result)
        )
dispatch = core.parent / "binding_api.rs"
dispatch_source = "// Generated by packages/npm/scripts/generate-api.py.\n" + r'''
use crate::{Bot, MediaTarget, Result, SdkError, Sendable, MessageRequest};
use serde_json::Value;

pub struct MediaTargetInput { kind: String, id: String }
impl MediaTargetInput {
    pub fn parse(value: Value) -> Result<Self> {
        let field = |key| value.get(key).and_then(Value::as_str).map(str::to_owned).ok_or_else(|| SdkError::InvalidInput("media target requires kind and id".into()));
        Ok(Self { kind: field("kind")?, id: field("id")? })
    }
    pub fn target(&self) -> Result<MediaTarget<'_>> {
        match self.kind.as_str() {
            "user" => Ok(MediaTarget::C2c(&self.id)),
            "group" => Ok(MediaTarget::Group(&self.id)),
            _ => Err(SdkError::InvalidInput("media target kind must be user or group".into())),
        }
    }
}

pub fn sendable(value: Value) -> Result<Sendable> {
    match value {
        Value::String(text) => Ok(text.into()),
        Value::Object(_) => Ok(serde_json::from_value::<MessageRequest>(value)?.into()),
        _ => Err(SdkError::InvalidInput("message must be text or a MessageRequest object".into())),
    }
}

pub async fn call(client: &Bot, operation: &str, args: Vec<Value>) -> Result<Value> {
    let api = client.api();
    match operation {
''' + "\n".join(arms) + '\n_ => Err(SdkError::InvalidInput("unknown API operation".into())),\n}\n}\n'
lines = ["// Generated by scripts/generate-api.py.", "import type { Message } from './index';", "import type { UploadSource } from './upload-source';", "export interface MediaTarget { kind: 'user' | 'group'; id: string }", "export interface BotApi {"]
for group, methods in groups.items():
    lines.append(f"  {group}: {{")
    for name, types in methods:
        lines.append(f"    {name}({', '.join(types)}): Promise<any>;")
    lines.append("  };")
lines += [
    "}",
    "export function createApi(",
    "  call: (operation: string, args: unknown[]) => Promise<unknown>,",
    "  uploadMaterial: BotApi['materials']['upload'],",
    "  media: Pick<BotApi['messages'], 'uploadMedia' | 'sendMedia'>,",
    "): BotApi {",
    "  const invoke = (operation: string, args: unknown[]) => call(operation, args.map(value => value === undefined ? null : value));",
    "  return {",
]
for group, methods in groups.items():
    lines.append(f"    {group}: {{")
    for name, _ in methods:
        if (group, name) in binary_methods:
            _, binding = binary_methods[group, name]
            lines.append(f"      {name}: {binding},")
        else:
            lines.append(f"      {name}: (...args: any[]) => invoke('{group}.{name}', args),")
    lines.append("    },")
lines += ["  };", "}"]
emit(root / "src" / "api.ts", "\n".join(lines) + "\n")

python_lines = [
    "# Generated by packages/npm/scripts/generate-api.py.",
    '"""Grouped QQ API methods shared with the Rust and Node.js SDKs."""',
    "",
    "from __future__ import annotations",
    "",
    "from typing import Any, Literal, TypedDict",
    "",
    "from ._native import Bot as _NativeBot",
    "",
    'Message = str | dict[str, Any]',
    "",
    "",
    "class MediaTarget(TypedDict):",
    '    kind: Literal["user", "group"]',
    "    id: str",
    "",
    "",
    "class _ApiGroup:",
    "    def __init__(self, client: _NativeBot) -> None:",
    "        self._client = client",
]
for group, methods in python_groups.items():
    group_class = f"{group.title()}Methods"
    python_lines.extend(["", "", f"class {group_class}(_ApiGroup):"])
    for index, (name, params, invoke, result) in enumerate(methods):
        if index:
            python_lines.append("")
        signature = f"self, {params}" if params else "self"
        if len(f"    async def {name}({signature}) -> {result}:") > 99:
            python_lines.extend([f"    async def {name}(", "        self,"])
            python_lines.extend(f"        {param}," for param in re.split(r", (?=\w+:)", params))
            python_lines.append(f"    ) -> {result}:")
        else:
            python_lines.append(f"    async def {name}({signature}) -> {result}:")
        python_lines.append(f"        return await {invoke}")
    if group == "messages":
        python_lines.extend("""
    async def _upload_media(
        self, target: MediaTarget, file_type: int, source: bytes | str, srv_send_msg: bool
    ) -> dict[str, Any]:
        if isinstance(source, str):
            return await self.upload_media_url(target, file_type, source, srv_send_msg)
        return await self._client._upload_media(
            target["kind"], target["id"], file_type, source, srv_send_msg
        )

    async def _send_media(
        self, target: MediaTarget, file_type: int, source: bytes | str, message: Message
    ) -> dict[str, Any]:
        if isinstance(source, str):
            return await self.send_media_url(target, file_type, source, message)
        return await self._client._send_media(
            target["kind"], target["id"], file_type, source, message
        )
""".rstrip().splitlines())
python_lines.extend(["", "", "class BotApi:"])
for group in python_groups:
    python_lines.append(f"    {group}: {group.title()}Methods")
python_lines.extend(["", "    def __init__(self, client: _NativeBot) -> None:"])
for group in python_groups:
    python_lines.append(f"        self.{group} = {group.title()}Methods(client)")
python_api = root.parent / "python" / "python" / "qqbot_sdk_py" / "api.py"
emit(python_api, "\n".join(python_lines) + "\n")

formatted = subprocess.run(
    ["rustfmt", "--edition", "2024", "--emit", "stdout"],
    input=dispatch_source,
    capture_output=True,
    text=True,
    check=True,
).stdout
emit(dispatch, formatted)

if stale:
    raise SystemExit("Regenerate SDK bindings: " + ", ".join(map(str, stale)))
