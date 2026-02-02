# Traits & Enums Konsolidierung - Vergleich

## Status: ✅ Domain-Version ist vollständig und erweitert

### Enums

| Enum | app/ | domain/ | Status |
|------|------|---------|--------|
| `Rotation` | ✅ | ✅ | ✅ Identisch |
| `FlipDirection` | ✅ | ✅ | ✅ Identisch |
| `RotationMode` | ❌ | ✅ | ℹ️ Extra in domain (fine rotation support) |
| `InterpolationQuality` | ❌ | ✅ | ℹ️ Extra in domain (quality control) |
| `DocumentKind` | ✅ | ✅ | ✅ Identisch (in content.rs) |

### Structs

| Struct | app/ | domain/ | Status |
|--------|------|---------|--------|
| `TransformState` | ✅ `rotation: Rotation` | ✅ `rotation: RotationMode` | ⚠️ Domain erweitert (RotationMode statt Rotation) |
| `RenderOutput` | ✅ | ✅ | ✅ Identisch |
| `DocumentInfo` | ✅ | ✅ | ✅ Identisch |

### Traits

#### Renderable

| Methode | app/ | domain/ | Status |
|---------|------|---------|--------|
| `render()` | ✅ | ✅ | ✅ Identisch |
| `info()` | ✅ | ✅ | ✅ Identisch |

#### Transformable

| Methode | app/ | domain/ | Status |
|---------|------|---------|--------|
| `rotate()` | ✅ | ✅ | ✅ Identisch |
| `flip()` | ✅ | ✅ | ✅ Identisch |
| `transform_state()` | ✅ | ✅ | ✅ Identisch |
| `rotate_fine()` | ❌ | ✅ | ℹ️ Extra in domain (default impl) |
| `reset_fine_rotation()` | ❌ | ✅ | ℹ️ Extra in domain (default impl) |
| `set_interpolation_quality()` | ❌ | ✅ | ℹ️ Extra in domain (default impl) |

#### MultiPage

| Methode | app/ | domain/ | Status |
|---------|------|---------|--------|
| `page_count()` | ✅ | ✅ | ✅ Identisch |
| `current_page()` | ✅ | ✅ | ✅ Identisch |
| `go_to_page()` | ✅ | ✅ | ✅ Identisch |

#### MultiPageThumbnails

| Methode | app/ | domain/ | Status |
|---------|------|---------|--------|
| `get_thumbnail()` | ✅ `Option<Handle>` | ✅ `Result<Option<Handle>>` | ⚠️ Domain hat error handling |
| `thumbnails_ready()` | ✅ | ✅ | ✅ Identisch |
| `thumbnails_loaded()` | ✅ | ✅ | ✅ Identisch |
| `generate_thumbnail_page()` | ✅ `Option<usize>` | ✅ `Result<()>` | ⚠️ Domain hat error handling |
| `generate_all_thumbnails()` | ✅ | ✅ | ℹ️ Beide vorhanden |

### Unterschiede

#### 1. RotationMode (Domain-Erweiterung)

**App:**
```rust
pub struct TransformState {
    pub rotation: Rotation,  // Nur 90° Schritte
}
```

**Domain:**
```rust
pub enum RotationMode {
    Standard(Rotation),  // 90° Schritte
    Fine(f32),           // Beliebige Winkel
}

pub struct TransformState {
    pub rotation: RotationMode,  // Flexibel!
}
```

**Vorteil:** Domain unterstützt fine rotation (RasterDocument nutzt das)

#### 2. Transformable Erweiterungen

**Domain hat zusätzlich:**
- `rotate_fine()` - Für beliebige Rotationswinkel
- `reset_fine_rotation()` - Reset zu 90° Schritten
- `set_interpolation_quality()` - Qualitätskontrolle

**Default Implementierungen:** Alle haben Default-Impls (no-op), daher backward-compatible.

#### 3. Error Handling in Thumbnails

**App:**
```rust
fn get_thumbnail(&self, page: usize) -> Option<ImageHandle>;
```

**Domain:**
```rust
fn get_thumbnail(&mut self, page: usize) -> DocResult<Option<ImageHandle>>;
```

**Vorteil:** Domain kann Fehler melden statt silent failure.

## Kompatibilität

### ⚠️ Potenzielle Breaking Changes

1. **TransformState::rotation** ist jetzt `RotationMode` statt `Rotation`
   - Alt: `state.rotation == Rotation::Cw90`
   - Neu: `state.rotation == RotationMode::Standard(Rotation::Cw90)`

2. **MultiPageThumbnails Signatur** unterscheidet sich
   - Kann zu Kompilierungsfehlern führen wenn app/ Code darauf zugreift

### ✅ Lösungen

**Option 1:** `RotationMode` bietet `From<Rotation>` an:
```rust
impl From<Rotation> for RotationMode {
    fn from(rot: Rotation) -> Self {
        RotationMode::Standard(rot)
    }
}
```

**Option 2:** Helper-Methode in TransformState:
```rust
impl TransformState {
    pub fn standard_rotation(&self) -> Option<Rotation> {
        match self.rotation {
            RotationMode::Standard(rot) => Some(rot),
            _ => None,
        }
    }
}
```

## Entscheidung

✅ **Domain-Version nutzen - ist besser!**

**Begründung:**
- Alle app/ Features sind vorhanden
- Zusätzliche Features (fine rotation, interpolation)
- Besseres Error Handling
- Backward-compatible durch Default-Impls

**Aktion:**
- Keine Änderungen nötig
- Domain ist bereits vollständig
- Bei Integration: Helper für RotationMode-Kompatibilität hinzufügen falls nötig

## Nächster Schritt

Phase 2: Infrastructure Layer Migration
