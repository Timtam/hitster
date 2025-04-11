import { useContext, useEffect, useMemo, useState } from "react"
import type {
    ToastIdType,
    ToastOptions,
    ToastOptionsWithId,
    ToastPropsOmitBg,
} from "./types"

import type { ToastProps } from "react-bootstrap"
import ToastsContext from "./ToastsContext"

let toastId: ToastIdType = 0

const useToasts = () => {
    const [toastOptsQueue, setToastOptsQueue] = useState<ToastOptionsWithId[]>(
        [],
    )

    const ctx = useContext(ToastsContext)
    if (ctx === undefined) {
        throw Error(
            "`useToasts` must be used inside of a `ToastsProvider`, " +
                "otherwise it will not function correctly.",
        )
    }

    const api = useMemo(() => {
        const show = (toastOptions: ToastOptions<ToastProps>): ToastIdType => {
            const id = toastId++
            setToastOptsQueue((q) => [...q, { ...toastOptions, id }])
            return id
        }
        const hide = (id: ToastIdType) => {
            ctx.current?.hide(id)
        }

        const info = (
            toastOptions: ToastOptions<ToastPropsOmitBg>,
        ): ToastIdType => {
            return show(_withBg(toastOptions, "info"))
        }
        const primary = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "primary"))
        }
        const secondary = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "secondary"))
        }
        const success = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "success"))
        }
        const danger = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "danger"))
        }
        const warning = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "warning"))
        }
        const dark = (toastOptions: ToastOptions<ToastPropsOmitBg>) => {
            return show(_withBg(toastOptions, "dark"))
        }
        const light = (
            toastOptions: ToastOptions<ToastPropsOmitBg>,
        ): ToastIdType => {
            return show(_withBg(toastOptions, "light"))
        }

        const _withBg = (
            toastOptions: ToastOptions<ToastPropsOmitBg>,
            bg: ToastProps["bg"],
        ) => {
            const { toastProps } = toastOptions
            const toastPropsWithBg = { ...toastProps, bg }
            return {
                ...toastOptions,
                toastProps: toastPropsWithBg,
            }
        }
        return {
            show,
            hide,
            info,
            primary,
            secondary,
            success,
            danger,
            warning,
            dark,
            light,
        }
    }, [ctx])

    useEffect(() => {
        const { current } = ctx
        if (current !== null && toastOptsQueue.length) {
            toastOptsQueue.forEach((opts) => {
                current.show(opts)
            })

            setToastOptsQueue([])
        }
    }, [ctx, toastOptsQueue])

    return api
}

export default useToasts
