[**@miden-sdk/miden-wallet-adapter-miden**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-miden](../README.md) / MidenWindow

# Interface: MidenWindow

## Extends

- `Window`

## Indexable

\[`index`: `number`\]: `Window`

## Properties

### caches

> `readonly` **caches**: `CacheStorage`

Available only in secure contexts.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/caches)

#### Inherited from

`Window.caches`

***

### ~~clientInformation~~

> `readonly` **clientInformation**: `Navigator`

#### Deprecated

This is a legacy alias of `navigator`.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/navigator)

#### Inherited from

`Window.clientInformation`

***

### closed

> `readonly` **closed**: `boolean`

The **`Window.closed`** read-only property indicates whether the referenced window is closed or not.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/closed)

#### Inherited from

`Window.closed`

***

### cookieStore

> `readonly` **cookieStore**: `CookieStore`

The **`cookieStore`** read-only property of the Window interface returns a reference to the CookieStore object for the current document context.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/cookieStore)

#### Inherited from

`Window.cookieStore`

***

### crossOriginIsolated

> `readonly` **crossOriginIsolated**: `boolean`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/crossOriginIsolated)

#### Inherited from

`Window.crossOriginIsolated`

***

### crypto

> `readonly` **crypto**: `Crypto`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/crypto)

#### Inherited from

`Window.crypto`

***

### customElements

> `readonly` **customElements**: `CustomElementRegistry`

The **`customElements`** read-only property of the Window interface returns a reference to the CustomElementRegistry object, which can be used to register new custom elements and get information about previously registered custom elements.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/customElements)

#### Inherited from

`Window.customElements`

***

### devicePixelRatio

> `readonly` **devicePixelRatio**: `number`

The **`devicePixelRatio`** of Window interface returns the ratio of the resolution in _physical pixels_ to the resolution in _CSS pixels_ for the current display device.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/devicePixelRatio)

#### Inherited from

`Window.devicePixelRatio`

***

### document

> `readonly` **document**: `Document`

**`window.document`** returns a reference to the document contained in the window.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/document)

#### Inherited from

`Window.document`

***

### ~~event~~

> `readonly` **event**: `Event`

The read-only Window property **`event`** returns the Event which is currently being handled by the site's code.

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/event)

#### Inherited from

`Window.event`

***

### ~~external~~

> `readonly` **external**: `External`

The `external` property of the Window API returns an instance of the `External` interface, which was intended to contain functions related to adding external search providers to the browser.

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/external)

#### Inherited from

`Window.external`

***

### frameElement

> `readonly` **frameElement**: `Element`

The **`Window.frameElement`** property returns the element (such as iframe or object) in which the window is embedded.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/frameElement)

#### Inherited from

`Window.frameElement`

***

### frames

> `readonly` **frames**: `Window`

Returns the window itself, which is an array-like object, listing the direct sub-frames of the current window.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/frames)

#### Inherited from

`Window.frames`

***

### history

> `readonly` **history**: `History`

The `Window.history` read-only property returns a reference to the History object, which provides an interface for manipulating the browser _session history_ (pages visited in the tab or frame that the current page is loaded in).

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/history)

#### Inherited from

`Window.history`

***

### indexedDB

> `readonly` **indexedDB**: `IDBFactory`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/indexedDB)

#### Inherited from

`Window.indexedDB`

***

### innerHeight

> `readonly` **innerHeight**: `number`

The read-only **`innerHeight`** property of the including the height of the horizontal scroll bar, if present.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/innerHeight)

#### Inherited from

`Window.innerHeight`

***

### innerWidth

> `readonly` **innerWidth**: `number`

The read-only Window property **`innerWidth`** returns the interior width of the window in pixels (that is, the width of the window's layout viewport).

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/innerWidth)

#### Inherited from

`Window.innerWidth`

***

### isSecureContext

> `readonly` **isSecureContext**: `boolean`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/isSecureContext)

#### Inherited from

`Window.isSecureContext`

***

### length

> `readonly` **length**: `number`

Returns the number of frames (either frame or A number.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/length)

#### Inherited from

`Window.length`

***

### localStorage

> `readonly` **localStorage**: `Storage`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/localStorage)

#### Inherited from

`Window.localStorage`

***

### locationbar

> `readonly` **locationbar**: `BarProp`

Returns the `locationbar` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/locationbar)

#### Inherited from

`Window.locationbar`

***

### menubar

> `readonly` **menubar**: `BarProp`

Returns the `menubar` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/menubar)

#### Inherited from

`Window.menubar`

***

### miden?

> `optional` **miden**: [`MidenWallet`](MidenWallet.md)

***

### midenWallet?

> `optional` **midenWallet**: [`MidenWallet`](MidenWallet.md)

***

### name

> **name**: `string`

The `Window.name` property gets/sets the name of the window's browsing context.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/name)

#### Inherited from

`Window.name`

***

### navigator

> `readonly` **navigator**: `Navigator`

The **`Window.navigator`** read-only property returns a reference to the Navigator object, which has methods and properties about the application running the script.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/navigator)

#### Inherited from

`Window.navigator`

***

### onabort()

> **onabort**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/abort_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`UIEvent`

#### Returns

`any`

#### Inherited from

`Window.onabort`

***

### onafterprint()

> **onafterprint**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/afterprint_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onafterprint`

***

### onanimationcancel()

> **onanimationcancel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationcancel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`AnimationEvent`

#### Returns

`any`

#### Inherited from

`Window.onanimationcancel`

***

### onanimationend()

> **onanimationend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`AnimationEvent`

#### Returns

`any`

#### Inherited from

`Window.onanimationend`

***

### onanimationiteration()

> **onanimationiteration**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationiteration_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`AnimationEvent`

#### Returns

`any`

#### Inherited from

`Window.onanimationiteration`

***

### onanimationstart()

> **onanimationstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`AnimationEvent`

#### Returns

`any`

#### Inherited from

`Window.onanimationstart`

***

### onauxclick()

> **onauxclick**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/auxclick_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onauxclick`

***

### onbeforeinput()

> **onbeforeinput**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/beforeinput_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`InputEvent`

#### Returns

`any`

#### Inherited from

`Window.onbeforeinput`

***

### onbeforematch()

> **onbeforematch**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/beforematch_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onbeforematch`

***

### onbeforeprint()

> **onbeforeprint**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/beforeprint_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onbeforeprint`

***

### onbeforetoggle()

> **onbeforetoggle**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/beforetoggle_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ToggleEvent`

#### Returns

`any`

#### Inherited from

`Window.onbeforetoggle`

***

### onbeforeunload()

> **onbeforeunload**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/beforeunload_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`BeforeUnloadEvent`

#### Returns

`any`

#### Inherited from

`Window.onbeforeunload`

***

### onblur()

> **onblur**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/blur_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`FocusEvent`

#### Returns

`any`

#### Inherited from

`Window.onblur`

***

### oncancel()

> **oncancel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLDialogElement/cancel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncancel`

***

### oncanplay()

> **oncanplay**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/canplay_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncanplay`

***

### oncanplaythrough()

> **oncanplaythrough**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/canplaythrough_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncanplaythrough`

***

### onchange()

> **onchange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/change_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onchange`

***

### onclick()

> **onclick**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/click_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onclick`

***

### onclose()

> **onclose**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLDialogElement/close_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onclose`

***

### oncontextlost()

> **oncontextlost**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLCanvasElement/contextlost_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncontextlost`

***

### oncontextmenu()

> **oncontextmenu**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/contextmenu_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.oncontextmenu`

***

### oncontextrestored()

> **oncontextrestored**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLCanvasElement/contextrestored_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncontextrestored`

***

### oncopy()

> **oncopy**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/copy_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ClipboardEvent`

#### Returns

`any`

#### Inherited from

`Window.oncopy`

***

### oncuechange()

> **oncuechange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLTrackElement/cuechange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oncuechange`

***

### oncut()

> **oncut**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/cut_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ClipboardEvent`

#### Returns

`any`

#### Inherited from

`Window.oncut`

***

### ondblclick()

> **ondblclick**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/dblclick_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.ondblclick`

***

### ondevicemotion()

> **ondevicemotion**: (`this`, `ev`) => `any`

Available only in secure contexts.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/devicemotion_event)

#### Parameters

##### this

`Window`

##### ev

`DeviceMotionEvent`

#### Returns

`any`

#### Inherited from

`Window.ondevicemotion`

***

### ondeviceorientation()

> **ondeviceorientation**: (`this`, `ev`) => `any`

Available only in secure contexts.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/deviceorientation_event)

#### Parameters

##### this

`Window`

##### ev

`DeviceOrientationEvent`

#### Returns

`any`

#### Inherited from

`Window.ondeviceorientation`

***

### ondeviceorientationabsolute()

> **ondeviceorientationabsolute**: (`this`, `ev`) => `any`

Available only in secure contexts.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/deviceorientationabsolute_event)

#### Parameters

##### this

`Window`

##### ev

`DeviceOrientationEvent`

#### Returns

`any`

#### Inherited from

`Window.ondeviceorientationabsolute`

***

### ondrag()

> **ondrag**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/drag_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondrag`

***

### ondragend()

> **ondragend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/dragend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondragend`

***

### ondragenter()

> **ondragenter**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/dragenter_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondragenter`

***

### ondragleave()

> **ondragleave**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/dragleave_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondragleave`

***

### ondragover()

> **ondragover**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/dragover_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondragover`

***

### ondragstart()

> **ondragstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/dragstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondragstart`

***

### ondrop()

> **ondrop**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/drop_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`DragEvent`

#### Returns

`any`

#### Inherited from

`Window.ondrop`

***

### ondurationchange()

> **ondurationchange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/durationchange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.ondurationchange`

***

### onemptied()

> **onemptied**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/emptied_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onemptied`

***

### onended()

> **onended**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/ended_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onended`

***

### onerror

> **onerror**: `OnErrorEventHandlerNonNull`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/error_event)

#### Inherited from

`Window.onerror`

***

### onfocus()

> **onfocus**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/focus_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`FocusEvent`

#### Returns

`any`

#### Inherited from

`Window.onfocus`

***

### onformdata()

> **onformdata**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLFormElement/formdata_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`FormDataEvent`

#### Returns

`any`

#### Inherited from

`Window.onformdata`

***

### ongamepadconnected()

> **ongamepadconnected**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/gamepadconnected_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`GamepadEvent`

#### Returns

`any`

#### Inherited from

`Window.ongamepadconnected`

***

### ongamepaddisconnected()

> **ongamepaddisconnected**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/gamepaddisconnected_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`GamepadEvent`

#### Returns

`any`

#### Inherited from

`Window.ongamepaddisconnected`

***

### ongotpointercapture()

> **ongotpointercapture**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/gotpointercapture_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.ongotpointercapture`

***

### onhashchange()

> **onhashchange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/hashchange_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`HashChangeEvent`

#### Returns

`any`

#### Inherited from

`Window.onhashchange`

***

### oninput()

> **oninput**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/input_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oninput`

***

### oninvalid()

> **oninvalid**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLInputElement/invalid_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.oninvalid`

***

### onkeydown()

> **onkeydown**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/keydown_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`KeyboardEvent`

#### Returns

`any`

#### Inherited from

`Window.onkeydown`

***

### ~~onkeypress()~~

> **onkeypress**: (`this`, `ev`) => `any`

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`KeyboardEvent`

#### Returns

`any`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/keypress_event)

#### Inherited from

`Window.onkeypress`

***

### onkeyup()

> **onkeyup**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/keyup_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`KeyboardEvent`

#### Returns

`any`

#### Inherited from

`Window.onkeyup`

***

### onlanguagechange()

> **onlanguagechange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/languagechange_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onlanguagechange`

***

### onload()

> **onload**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/load_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onload`

***

### onloadeddata()

> **onloadeddata**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/loadeddata_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onloadeddata`

***

### onloadedmetadata()

> **onloadedmetadata**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/loadedmetadata_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onloadedmetadata`

***

### onloadstart()

> **onloadstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/loadstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onloadstart`

***

### onlostpointercapture()

> **onlostpointercapture**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/lostpointercapture_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onlostpointercapture`

***

### onmessage()

> **onmessage**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/message_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`MessageEvent`

#### Returns

`any`

#### Inherited from

`Window.onmessage`

***

### onmessageerror()

> **onmessageerror**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/messageerror_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`MessageEvent`

#### Returns

`any`

#### Inherited from

`Window.onmessageerror`

***

### onmousedown()

> **onmousedown**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mousedown_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmousedown`

***

### onmouseenter()

> **onmouseenter**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mouseenter_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmouseenter`

***

### onmouseleave()

> **onmouseleave**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mouseleave_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmouseleave`

***

### onmousemove()

> **onmousemove**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mousemove_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmousemove`

***

### onmouseout()

> **onmouseout**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mouseout_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmouseout`

***

### onmouseover()

> **onmouseover**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mouseover_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmouseover`

***

### onmouseup()

> **onmouseup**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/mouseup_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`MouseEvent`

#### Returns

`any`

#### Inherited from

`Window.onmouseup`

***

### onoffline()

> **onoffline**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/offline_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onoffline`

***

### ononline()

> **ononline**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/online_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.ononline`

***

### ~~onorientationchange()~~

> **onorientationchange**: (`this`, `ev`) => `any`

#### Parameters

##### this

`Window`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/orientationchange_event)

#### Inherited from

`Window.onorientationchange`

***

### onpagehide()

> **onpagehide**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/pagehide_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PageTransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.onpagehide`

***

### onpagereveal()

> **onpagereveal**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/pagereveal_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PageRevealEvent`

#### Returns

`any`

#### Inherited from

`Window.onpagereveal`

***

### onpageshow()

> **onpageshow**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/pageshow_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PageTransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.onpageshow`

***

### onpageswap()

> **onpageswap**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/pageswap_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PageSwapEvent`

#### Returns

`any`

#### Inherited from

`Window.onpageswap`

***

### onpaste()

> **onpaste**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/paste_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ClipboardEvent`

#### Returns

`any`

#### Inherited from

`Window.onpaste`

***

### onpause()

> **onpause**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/pause_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onpause`

***

### onplay()

> **onplay**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/play_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onplay`

***

### onplaying()

> **onplaying**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/playing_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onplaying`

***

### onpointercancel()

> **onpointercancel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointercancel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointercancel`

***

### onpointerdown()

> **onpointerdown**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerdown_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerdown`

***

### onpointerenter()

> **onpointerenter**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerenter_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerenter`

***

### onpointerleave()

> **onpointerleave**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerleave_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerleave`

***

### onpointermove()

> **onpointermove**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointermove_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointermove`

***

### onpointerout()

> **onpointerout**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerout_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerout`

***

### onpointerover()

> **onpointerover**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerover_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerover`

***

### onpointerrawupdate()

> **onpointerrawupdate**: (`this`, `ev`) => `any`

Available only in secure contexts.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerrawupdate_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onpointerrawupdate`

***

### onpointerup()

> **onpointerup**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/pointerup_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`PointerEvent`

#### Returns

`any`

#### Inherited from

`Window.onpointerup`

***

### onpopstate()

> **onpopstate**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/popstate_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PopStateEvent`

#### Returns

`any`

#### Inherited from

`Window.onpopstate`

***

### onprogress()

> **onprogress**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/progress_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ProgressEvent`

#### Returns

`any`

#### Inherited from

`Window.onprogress`

***

### onratechange()

> **onratechange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/ratechange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onratechange`

***

### onrejectionhandled()

> **onrejectionhandled**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/rejectionhandled_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PromiseRejectionEvent`

#### Returns

`any`

#### Inherited from

`Window.onrejectionhandled`

***

### onreset()

> **onreset**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLFormElement/reset_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onreset`

***

### onresize()

> **onresize**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLVideoElement/resize_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`UIEvent`

#### Returns

`any`

#### Inherited from

`Window.onresize`

***

### onscroll()

> **onscroll**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Document/scroll_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onscroll`

***

### onscrollend()

> **onscrollend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Document/scrollend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onscrollend`

***

### onsecuritypolicyviolation()

> **onsecuritypolicyviolation**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Document/securitypolicyviolation_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`SecurityPolicyViolationEvent`

#### Returns

`any`

#### Inherited from

`Window.onsecuritypolicyviolation`

***

### onseeked()

> **onseeked**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/seeked_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onseeked`

***

### onseeking()

> **onseeking**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/seeking_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onseeking`

***

### onselect()

> **onselect**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLInputElement/select_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onselect`

***

### onselectionchange()

> **onselectionchange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Document/selectionchange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onselectionchange`

***

### onselectstart()

> **onselectstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Node/selectstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onselectstart`

***

### onslotchange()

> **onslotchange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLSlotElement/slotchange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onslotchange`

***

### onstalled()

> **onstalled**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/stalled_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onstalled`

***

### onstorage()

> **onstorage**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/storage_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`StorageEvent`

#### Returns

`any`

#### Inherited from

`Window.onstorage`

***

### onsubmit()

> **onsubmit**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLFormElement/submit_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`SubmitEvent`

#### Returns

`any`

#### Inherited from

`Window.onsubmit`

***

### onsuspend()

> **onsuspend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/suspend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onsuspend`

***

### ontimeupdate()

> **ontimeupdate**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/timeupdate_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.ontimeupdate`

***

### ontoggle()

> **ontoggle**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLElement/toggle_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`ToggleEvent`

#### Returns

`any`

#### Inherited from

`Window.ontoggle`

***

### ontouchcancel()?

> `optional` **ontouchcancel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/touchcancel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TouchEvent`

#### Returns

`any`

#### Inherited from

`Window.ontouchcancel`

***

### ontouchend()?

> `optional` **ontouchend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/touchend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TouchEvent`

#### Returns

`any`

#### Inherited from

`Window.ontouchend`

***

### ontouchmove()?

> `optional` **ontouchmove**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/touchmove_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TouchEvent`

#### Returns

`any`

#### Inherited from

`Window.ontouchmove`

***

### ontouchstart()?

> `optional` **ontouchstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/touchstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TouchEvent`

#### Returns

`any`

#### Inherited from

`Window.ontouchstart`

***

### ontransitioncancel()

> **ontransitioncancel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/transitioncancel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.ontransitioncancel`

***

### ontransitionend()

> **ontransitionend**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/transitionend_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.ontransitionend`

***

### ontransitionrun()

> **ontransitionrun**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/transitionrun_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.ontransitionrun`

***

### ontransitionstart()

> **ontransitionstart**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/transitionstart_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`TransitionEvent`

#### Returns

`any`

#### Inherited from

`Window.ontransitionstart`

***

### onunhandledrejection()

> **onunhandledrejection**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/unhandledrejection_event)

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`PromiseRejectionEvent`

#### Returns

`any`

#### Inherited from

`Window.onunhandledrejection`

***

### ~~onunload()~~

> **onunload**: (`this`, `ev`) => `any`

#### Parameters

##### this

`WindowEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/unload_event)

#### Inherited from

`Window.onunload`

***

### onvolumechange()

> **onvolumechange**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/volumechange_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onvolumechange`

***

### onwaiting()

> **onwaiting**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/HTMLMediaElement/waiting_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Inherited from

`Window.onwaiting`

***

### ~~onwebkitanimationend()~~

> **onwebkitanimationend**: (`this`, `ev`) => `any`

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

This is a legacy alias of `onanimationend`.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationend_event)

#### Inherited from

`Window.onwebkitanimationend`

***

### ~~onwebkitanimationiteration()~~

> **onwebkitanimationiteration**: (`this`, `ev`) => `any`

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

This is a legacy alias of `onanimationiteration`.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationiteration_event)

#### Inherited from

`Window.onwebkitanimationiteration`

***

### ~~onwebkitanimationstart()~~

> **onwebkitanimationstart**: (`this`, `ev`) => `any`

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

This is a legacy alias of `onanimationstart`.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/animationstart_event)

#### Inherited from

`Window.onwebkitanimationstart`

***

### ~~onwebkittransitionend()~~

> **onwebkittransitionend**: (`this`, `ev`) => `any`

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`Event`

#### Returns

`any`

#### Deprecated

This is a legacy alias of `ontransitionend`.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/transitionend_event)

#### Inherited from

`Window.onwebkittransitionend`

***

### onwheel()

> **onwheel**: (`this`, `ev`) => `any`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Element/wheel_event)

#### Parameters

##### this

`GlobalEventHandlers`

##### ev

`WheelEvent`

#### Returns

`any`

#### Inherited from

`Window.onwheel`

***

### opener

> **opener**: `any`

The Window interface's **`opener`** property returns a reference to the window that opened the window, either with Window.open, or by navigating a link with a `target` attribute.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/opener)

#### Inherited from

`Window.opener`

***

### ~~orientation~~

> `readonly` **orientation**: `number`

Returns the orientation in degrees (in 90-degree increments) of the viewport relative to the device's natural orientation.

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/orientation)

#### Inherited from

`Window.orientation`

***

### origin

> `readonly` **origin**: `string`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/origin)

#### Inherited from

`Window.origin`

***

### originAgentCluster

> `readonly` **originAgentCluster**: `boolean`

The **`originAgentCluster`** read-only property of the Window interface returns `true` if this window belongs to an _origin-keyed agent cluster_: this means that the operating system has provided dedicated resources (for example an operating system process) to this window's origin that are not shared with windows from other origins.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/originAgentCluster)

#### Inherited from

`Window.originAgentCluster`

***

### outerHeight

> `readonly` **outerHeight**: `number`

The **`Window.outerHeight`** read-only property returns the height in pixels of the whole browser window, including any sidebar, window chrome, and window-resizing borders/handles.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/outerHeight)

#### Inherited from

`Window.outerHeight`

***

### outerWidth

> `readonly` **outerWidth**: `number`

**`Window.outerWidth`** read-only property returns the width of the outside of the browser window.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/outerWidth)

#### Inherited from

`Window.outerWidth`

***

### pageXOffset

> `readonly` **pageXOffset**: `number`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollX)

#### Inherited from

`Window.pageXOffset`

***

### pageYOffset

> `readonly` **pageYOffset**: `number`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollY)

#### Inherited from

`Window.pageYOffset`

***

### parent

> `readonly` **parent**: `Window`

The **`Window.parent`** property is a reference to the parent of the current window or subframe.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/parent)

#### Inherited from

`Window.parent`

***

### performance

> `readonly` **performance**: `Performance`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/performance)

#### Inherited from

`Window.performance`

***

### personalbar

> `readonly` **personalbar**: `BarProp`

Returns the `personalbar` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/personalbar)

#### Inherited from

`Window.personalbar`

***

### screen

> `readonly` **screen**: `Screen`

The Window property **`screen`** returns a reference to the screen object associated with the window.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/screen)

#### Inherited from

`Window.screen`

***

### screenLeft

> `readonly` **screenLeft**: `number`

The **`Window.screenLeft`** read-only property returns the horizontal distance, in CSS pixels, from the left border of the user's browser viewport to the left side of the screen.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/screenLeft)

#### Inherited from

`Window.screenLeft`

***

### screenTop

> `readonly` **screenTop**: `number`

The **`Window.screenTop`** read-only property returns the vertical distance, in CSS pixels, from the top border of the user's browser viewport to the top side of the screen.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/screenTop)

#### Inherited from

`Window.screenTop`

***

### screenX

> `readonly` **screenX**: `number`

The **`Window.screenX`** read-only property returns the horizontal distance, in CSS pixels, of the left border of the user's browser viewport to the left side of the screen.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/screenX)

#### Inherited from

`Window.screenX`

***

### screenY

> `readonly` **screenY**: `number`

The **`Window.screenY`** read-only property returns the vertical distance, in CSS pixels, of the top border of the user's browser viewport to the top edge of the screen.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/screenY)

#### Inherited from

`Window.screenY`

***

### scrollbars

> `readonly` **scrollbars**: `BarProp`

Returns the `scrollbars` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollbars)

#### Inherited from

`Window.scrollbars`

***

### scrollX

> `readonly` **scrollX**: `number`

The read-only **`scrollX`** property of the Window interface returns the number of pixels by which the document is currently scrolled horizontally.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollX)

#### Inherited from

`Window.scrollX`

***

### scrollY

> `readonly` **scrollY**: `number`

The read-only **`scrollY`** property of the Window interface returns the number of pixels by which the document is currently scrolled vertically.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollY)

#### Inherited from

`Window.scrollY`

***

### self

> `readonly` **self**: `Window` & *typeof* `globalThis`

The **`Window.self`** read-only property returns the window itself, as a WindowProxy.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/self)

#### Inherited from

`Window.self`

***

### sessionStorage

> `readonly` **sessionStorage**: `Storage`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/sessionStorage)

#### Inherited from

`Window.sessionStorage`

***

### speechSynthesis

> `readonly` **speechSynthesis**: `SpeechSynthesis`

The `speechSynthesis` read-only property of the Window object returns a SpeechSynthesis object, which is the entry point into using Web Speech API speech synthesis functionality.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/speechSynthesis)

#### Inherited from

`Window.speechSynthesis`

***

### ~~status~~

> **status**: `string`

The **`status`** property of the bar at the bottom of the browser window.

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/status)

#### Inherited from

`Window.status`

***

### statusbar

> `readonly` **statusbar**: `BarProp`

Returns the `statusbar` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/statusbar)

#### Inherited from

`Window.statusbar`

***

### toolbar

> `readonly` **toolbar**: `BarProp`

Returns the `toolbar` object.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/toolbar)

#### Inherited from

`Window.toolbar`

***

### top

> `readonly` **top**: `Window`

Returns a reference to the topmost window in the window hierarchy.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/top)

#### Inherited from

`Window.top`

***

### visualViewport

> `readonly` **visualViewport**: `VisualViewport`

The **`visualViewport`** read-only property of the Window interface returns a VisualViewport object representing the visual viewport for a given window, or `null` if current document is not fully active.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/visualViewport)

#### Inherited from

`Window.visualViewport`

***

### window

> `readonly` **window**: `Window` & *typeof* `globalThis`

The **`window`** property of a Window object points to the window object itself.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/window)

#### Inherited from

`Window.window`

## Accessors

### location

#### Get Signature

> **get** **location**(): `Location`

The **`Window.location`** read-only property returns a Location object with information about the current location of the document.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/location)

##### Returns

`Location`

#### Set Signature

> **set** **location**(`href`): `void`

##### Parameters

###### href

`string`

##### Returns

`void`

#### Inherited from

`Window.location`

## Methods

### addEventListener()

#### Call Signature

> **addEventListener**\<`K`\>(`type`, `listener`, `options?`): `void`

The **`addEventListener()`** method of the EventTarget interface sets up a function that will be called whenever the specified event is delivered to the target.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/addEventListener)

##### Type Parameters

###### K

`K` *extends* keyof `WindowEventMap`

##### Parameters

###### type

`K`

###### listener

(`this`, `ev`) => `any`

###### options?

`boolean` | `AddEventListenerOptions`

##### Returns

`void`

##### Inherited from

`Window.addEventListener`

#### Call Signature

> **addEventListener**(`type`, `listener`, `options?`): `void`

The **`addEventListener()`** method of the EventTarget interface sets up a function that will be called whenever the specified event is delivered to the target.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/addEventListener)

##### Parameters

###### type

`string`

###### listener

`EventListenerOrEventListenerObject`

###### options?

`boolean` | `AddEventListenerOptions`

##### Returns

`void`

##### Inherited from

`Window.addEventListener`

***

### alert()

> **alert**(`message?`): `void`

`window.alert()` instructs the browser to display a dialog with an optional message, and to wait until the user dismisses the dialog.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/alert)

#### Parameters

##### message?

`any`

#### Returns

`void`

#### Inherited from

`Window.alert`

***

### atob()

> **atob**(`data`): `string`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/atob)

#### Parameters

##### data

`string`

#### Returns

`string`

#### Inherited from

`Window.atob`

***

### ~~blur()~~

> **blur**(): `void`

The **`Window.blur()`** method does nothing.

#### Returns

`void`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/blur)

#### Inherited from

`Window.blur`

***

### btoa()

> **btoa**(`data`): `string`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/btoa)

#### Parameters

##### data

`string`

#### Returns

`string`

#### Inherited from

`Window.btoa`

***

### cancelAnimationFrame()

> **cancelAnimationFrame**(`handle`): `void`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/DedicatedWorkerGlobalScope/cancelAnimationFrame)

#### Parameters

##### handle

`number`

#### Returns

`void`

#### Inherited from

`Window.cancelAnimationFrame`

***

### cancelIdleCallback()

> **cancelIdleCallback**(`handle`): `void`

The **`window.cancelIdleCallback()`** method cancels a callback previously scheduled with window.requestIdleCallback().

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/cancelIdleCallback)

#### Parameters

##### handle

`number`

#### Returns

`void`

#### Inherited from

`Window.cancelIdleCallback`

***

### ~~captureEvents()~~

> **captureEvents**(): `void`

The **`Window.captureEvents()`** method does nothing.

#### Returns

`void`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/captureEvents)

#### Inherited from

`Window.captureEvents`

***

### clearInterval()

> **clearInterval**(`id`): `void`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/clearInterval)

#### Parameters

##### id

`number`

#### Returns

`void`

#### Inherited from

`Window.clearInterval`

***

### clearTimeout()

> **clearTimeout**(`id`): `void`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/clearTimeout)

#### Parameters

##### id

`number`

#### Returns

`void`

#### Inherited from

`Window.clearTimeout`

***

### close()

> **close**(): `void`

The **`Window.close()`** method closes the current window, or the window on which it was called.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/close)

#### Returns

`void`

#### Inherited from

`Window.close`

***

### confirm()

> **confirm**(`message?`): `boolean`

`window.confirm()` instructs the browser to display a dialog with an optional message, and to wait until the user either confirms or cancels the dialog.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/confirm)

#### Parameters

##### message?

`string`

#### Returns

`boolean`

#### Inherited from

`Window.confirm`

***

### createImageBitmap()

#### Call Signature

> **createImageBitmap**(`image`, `options?`): `Promise`\<`ImageBitmap`\>

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/createImageBitmap)

##### Parameters

###### image

`ImageBitmapSource`

###### options?

`ImageBitmapOptions`

##### Returns

`Promise`\<`ImageBitmap`\>

##### Inherited from

`Window.createImageBitmap`

#### Call Signature

> **createImageBitmap**(`image`, `sx`, `sy`, `sw`, `sh`, `options?`): `Promise`\<`ImageBitmap`\>

##### Parameters

###### image

`ImageBitmapSource`

###### sx

`number`

###### sy

`number`

###### sw

`number`

###### sh

`number`

###### options?

`ImageBitmapOptions`

##### Returns

`Promise`\<`ImageBitmap`\>

##### Inherited from

`Window.createImageBitmap`

***

### dispatchEvent()

> **dispatchEvent**(`event`): `boolean`

The **`dispatchEvent()`** method of the EventTarget sends an Event to the object, (synchronously) invoking the affected event listeners in the appropriate order.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/dispatchEvent)

#### Parameters

##### event

`Event`

#### Returns

`boolean`

#### Inherited from

`Window.dispatchEvent`

***

### fetch()

> **fetch**(`input`, `init?`): `Promise`\<`Response`\>

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/fetch)

#### Parameters

##### input

`URL` | `RequestInfo`

##### init?

`RequestInit`

#### Returns

`Promise`\<`Response`\>

#### Inherited from

`Window.fetch`

***

### focus()

> **focus**(): `void`

Makes a request to bring the window to the front.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/focus)

#### Returns

`void`

#### Inherited from

`Window.focus`

***

### getComputedStyle()

> **getComputedStyle**(`elt`, `pseudoElt?`): `CSSStyleDeclaration`

The **`Window.getComputedStyle()`** method returns an object containing the values of all CSS properties of an element, after applying active stylesheets and resolving any basic computation those values may contain.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/getComputedStyle)

#### Parameters

##### elt

`Element`

##### pseudoElt?

`string`

#### Returns

`CSSStyleDeclaration`

#### Inherited from

`Window.getComputedStyle`

***

### getSelection()

> **getSelection**(): `Selection`

The **`getSelection()`** method of the Window interface returns the Selection object associated with the window's document, representing the range of text selected by the user or the current position of the caret.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/getSelection)

#### Returns

`Selection`

#### Inherited from

`Window.getSelection`

***

### matchMedia()

> **matchMedia**(`query`): `MediaQueryList`

The Window interface's **`matchMedia()`** method returns a new MediaQueryList object that can then be used to determine if the document matches the media query string, as well as to monitor the document to detect when it matches (or stops matching) that media query.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/matchMedia)

#### Parameters

##### query

`string`

#### Returns

`MediaQueryList`

#### Inherited from

`Window.matchMedia`

***

### moveBy()

> **moveBy**(`x`, `y`): `void`

The **`moveBy()`** method of the Window interface moves the current window by a specified amount.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/moveBy)

#### Parameters

##### x

`number`

##### y

`number`

#### Returns

`void`

#### Inherited from

`Window.moveBy`

***

### moveTo()

> **moveTo**(`x`, `y`): `void`

The **`moveTo()`** method of the Window interface moves the current window to the specified coordinates.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/moveTo)

#### Parameters

##### x

`number`

##### y

`number`

#### Returns

`void`

#### Inherited from

`Window.moveTo`

***

### open()

> **open**(`url?`, `target?`, `features?`): `Window`

The **`open()`** method of the `Window` interface loads a specified resource into a new or existing browsing context (that is, a tab, a window, or an iframe) under a specified name.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/open)

#### Parameters

##### url?

`string` | `URL`

##### target?

`string`

##### features?

`string`

#### Returns

`Window`

#### Inherited from

`Window.open`

***

### postMessage()

#### Call Signature

> **postMessage**(`message`, `targetOrigin`, `transfer?`): `void`

The **`window.postMessage()`** method safely enables cross-origin communication between Window objects; _e.g.,_ between a page and a pop-up that it spawned, or between a page and an iframe embedded within it.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/postMessage)

##### Parameters

###### message

`any`

###### targetOrigin

`string`

###### transfer?

`Transferable`[]

##### Returns

`void`

##### Inherited from

`Window.postMessage`

#### Call Signature

> **postMessage**(`message`, `options?`): `void`

##### Parameters

###### message

`any`

###### options?

`WindowPostMessageOptions`

##### Returns

`void`

##### Inherited from

`Window.postMessage`

***

### print()

> **print**(): `void`

Opens the print dialog to print the current document.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/print)

#### Returns

`void`

#### Inherited from

`Window.print`

***

### prompt()

> **prompt**(`message?`, `_default?`): `string`

`window.prompt()` instructs the browser to display a dialog with an optional message prompting the user to input some text, and to wait until the user either submits the text or cancels the dialog.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/prompt)

#### Parameters

##### message?

`string`

##### \_default?

`string`

#### Returns

`string`

#### Inherited from

`Window.prompt`

***

### queueMicrotask()

> **queueMicrotask**(`callback`): `void`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/queueMicrotask)

#### Parameters

##### callback

`VoidFunction`

#### Returns

`void`

#### Inherited from

`Window.queueMicrotask`

***

### ~~releaseEvents()~~

> **releaseEvents**(): `void`

Releases the window from trapping events of a specific type.

#### Returns

`void`

#### Deprecated

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/releaseEvents)

#### Inherited from

`Window.releaseEvents`

***

### removeEventListener()

#### Call Signature

> **removeEventListener**\<`K`\>(`type`, `listener`, `options?`): `void`

The **`removeEventListener()`** method of the EventTarget interface removes an event listener previously registered with EventTarget.addEventListener() from the target.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/removeEventListener)

##### Type Parameters

###### K

`K` *extends* keyof `WindowEventMap`

##### Parameters

###### type

`K`

###### listener

(`this`, `ev`) => `any`

###### options?

`boolean` | `EventListenerOptions`

##### Returns

`void`

##### Inherited from

`Window.removeEventListener`

#### Call Signature

> **removeEventListener**(`type`, `listener`, `options?`): `void`

The **`removeEventListener()`** method of the EventTarget interface removes an event listener previously registered with EventTarget.addEventListener() from the target.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/removeEventListener)

##### Parameters

###### type

`string`

###### listener

`EventListenerOrEventListenerObject`

###### options?

`boolean` | `EventListenerOptions`

##### Returns

`void`

##### Inherited from

`Window.removeEventListener`

***

### reportError()

> **reportError**(`e`): `void`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/reportError)

#### Parameters

##### e

`any`

#### Returns

`void`

#### Inherited from

`Window.reportError`

***

### requestAnimationFrame()

> **requestAnimationFrame**(`callback`): `number`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/DedicatedWorkerGlobalScope/requestAnimationFrame)

#### Parameters

##### callback

`FrameRequestCallback`

#### Returns

`number`

#### Inherited from

`Window.requestAnimationFrame`

***

### requestIdleCallback()

> **requestIdleCallback**(`callback`, `options?`): `number`

The **`window.requestIdleCallback()`** method queues a function to be called during a browser's idle periods.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/requestIdleCallback)

#### Parameters

##### callback

`IdleRequestCallback`

##### options?

`IdleRequestOptions`

#### Returns

`number`

#### Inherited from

`Window.requestIdleCallback`

***

### resizeBy()

> **resizeBy**(`x`, `y`): `void`

The **`Window.resizeBy()`** method resizes the current window by a specified amount.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/resizeBy)

#### Parameters

##### x

`number`

##### y

`number`

#### Returns

`void`

#### Inherited from

`Window.resizeBy`

***

### resizeTo()

> **resizeTo**(`width`, `height`): `void`

The **`Window.resizeTo()`** method dynamically resizes the window.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/resizeTo)

#### Parameters

##### width

`number`

##### height

`number`

#### Returns

`void`

#### Inherited from

`Window.resizeTo`

***

### scroll()

#### Call Signature

> **scroll**(`options?`): `void`

The **`Window.scroll()`** method scrolls the window to a particular place in the document.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scroll)

##### Parameters

###### options?

`ScrollToOptions`

##### Returns

`void`

##### Inherited from

`Window.scroll`

#### Call Signature

> **scroll**(`x`, `y`): `void`

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`void`

##### Inherited from

`Window.scroll`

***

### scrollBy()

#### Call Signature

> **scrollBy**(`options?`): `void`

The **`Window.scrollBy()`** method scrolls the document in the window by the given amount.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollBy)

##### Parameters

###### options?

`ScrollToOptions`

##### Returns

`void`

##### Inherited from

`Window.scrollBy`

#### Call Signature

> **scrollBy**(`x`, `y`): `void`

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`void`

##### Inherited from

`Window.scrollBy`

***

### scrollTo()

#### Call Signature

> **scrollTo**(`options?`): `void`

**`Window.scrollTo()`** scrolls to a particular set of coordinates in the document.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/scrollTo)

##### Parameters

###### options?

`ScrollToOptions`

##### Returns

`void`

##### Inherited from

`Window.scrollTo`

#### Call Signature

> **scrollTo**(`x`, `y`): `void`

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`void`

##### Inherited from

`Window.scrollTo`

***

### setInterval()

> **setInterval**(`handler`, `timeout?`, ...`arguments?`): `number`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/setInterval)

#### Parameters

##### handler

`TimerHandler`

##### timeout?

`number`

##### arguments?

...`any`[]

#### Returns

`number`

#### Inherited from

`Window.setInterval`

***

### setTimeout()

> **setTimeout**(`handler`, `timeout?`, ...`arguments?`): `number`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/setTimeout)

#### Parameters

##### handler

`TimerHandler`

##### timeout?

`number`

##### arguments?

...`any`[]

#### Returns

`number`

#### Inherited from

`Window.setTimeout`

***

### stop()

> **stop**(): `void`

The **`window.stop()`** stops further resource loading in the current browsing context, equivalent to the stop button in the browser.

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/stop)

#### Returns

`void`

#### Inherited from

`Window.stop`

***

### structuredClone()

> **structuredClone**\<`T`\>(`value`, `options?`): `T`

[MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/structuredClone)

#### Type Parameters

##### T

`T` = `any`

#### Parameters

##### value

`T`

##### options?

`StructuredSerializeOptions`

#### Returns

`T`

#### Inherited from

`Window.structuredClone`
