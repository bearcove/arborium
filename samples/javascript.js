import bind, { isCheckbox, isRadio, safeParseBoolean } from '../utils/bind'
import { evaluateLater } from '../evaluator'
import { directive } from '../directives'
import { mutateDom } from '../mutation'
import { nextTick } from '../nextTick'
import { isCloning } from '../clone'
import on from '../utils/on'

directive('model', (el, { modifiers, expression }, { effect, cleanup }) => {
    let scopeTarget = el

    if (modifiers.includes('parent')) {
        scopeTarget = el.parentNode
    }

    let evaluateGet = evaluateLater(scopeTarget, expression)
    let evaluateSet

    if (typeof expression === 'string') {
        evaluateSet = evaluateLater(scopeTarget, `${expression} = __placeholder`)
    } else if (typeof expression === 'function' && typeof expression() === 'string') {
        evaluateSet = evaluateLater(scopeTarget, `${expression()} = __placeholder`)
    } else {
        evaluateSet = () => {}
    }

    let getValue = () => {
        let result

        evaluateGet(value => result = value)

        return isGetterSetter(result) ? result.get() : result
    }

    let setValue = value => {
        let result

        evaluateGet(value => result = value)

        if (isGetterSetter(result)) {
            result.set(value)
        } else {
            evaluateSet(() => {}, {
                scope: { '__placeholder': value }
            })
        }
    }

    if (typeof expression === 'string' && el.type === 'radio') {
        // Radio buttons only work properly when they share a name attribute.
        // People might assume we take care of that for them, because
        // they already set a shared "x-model" attribute.
        mutateDom(() => {
            if (! el.hasAttribute('name')) el.setAttribute('name', expression)
        })
    }

    // If the element we are binding to is a select, a radio, or checkbox
    // we'll listen for the change event instead of the "input" event.
    let event = (el.tagName.toLowerCase() === 'select')
        || ['checkbox', 'radio'].includes(el.type)
        || modifiers.includes('lazy')
            ? 'change' : 'input'

    // We only want to register the event listener when we're not cloning, since the
    // mutation observer handles initializing the x-model directive already when
    // the element is inserted into the DOM. Otherwise we register it twice.
    let removeListener = isCloning ? () => {} : on(el, event, modifiers, (e) => {
        setValue(getInputValue(el, modifiers, e, getValue()))
    })

    if (modifiers.includes('fill'))
        if ([undefined, null, ''].includes(getValue())
            || (isCheckbox(el) && Array.isArray(getValue()))
            || (el.tagName.toLowerCase() === 'select' && el.multiple)) {
        setValue(
            getInputValue(el, modifiers, { target: el }, getValue())
        );
    }

    // Register the listener removal callback on the element, so that
    // in addition to the cleanup function, x-modelable may call it.
    // Also, make this a keyed object if we decide to reintroduce
    // "named modelables" some time in a future Alpine version.
    if (! el._x_removeModelListeners) el._x_removeModelListeners = {}
    el._x_removeModelListeners['default'] = removeListener

    cleanup(() => el._x_removeModelListeners['default']())

    // If the input/select/textarea element is linked to a form
    // we listen for the reset event on the parent form (the event
    // does not trigger on the single inputs) and update
    // on nextTick so the page doesn't end up out of sync
    if (el.form) {
        let removeResetListener = on(el.form, 'reset', [], (e) => {
            nextTick(() => el._x_model && el._x_model.set(getInputValue(el, modifiers, { target: el }, getValue())))
        })
        cleanup(() => removeResetListener())
    }

    // Allow programmatic overriding of x-model.
    el._x_model = {
        get() {
            return getValue()
        },
        set(value) {
            setValue(value)
        },
    }

    el._x_forceModelUpdate = (value) => {
        // If nested model key is undefined, set the default value to empty string.
        if (value === undefined && typeof expression === 'string' && expression.match(/\./)) value = ''

        // @todo: This is nasty
        window.fromModel = true
        mutateDom(() => bind(el, 'value', value))
        delete window.fromModel
    }

