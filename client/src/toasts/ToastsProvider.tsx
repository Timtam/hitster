import { forwardRef, JSX, useImperativeHandle, useRef, useState } from "react"
import { Toast, ToastContainer } from "react-bootstrap"
import { ToastIdType, ToastOptionsWithId, ToastsHandle } from "./types"

import type { ToastContainerProps } from "react-bootstrap"
import ToastsContext from "./ToastsContext"

const Toasts = forwardRef<
    ToastsHandle,
    { toastContainerProps?: ToastContainerProps; limit?: number }
>((props, ref) => {
    const [toasts, setToasts] = useState<ToastOptionsWithId[]>([])

    useImperativeHandle(ref, () => ({
        show: (toastOptionsWithId: ToastOptionsWithId) => {
            setToasts((state) => {
                const clone = [...state]
                clone.push(toastOptionsWithId)
                if (props.limit && clone.length > props.limit) {
                    clone.shift()
                }
                return clone
            })
        },

        hide: (id: ToastIdType) => {
            setToasts((state) => [...state].filter((t) => t.id !== id))
        },
    }))

    const { toastContainerProps } = props
    return (
        <ToastContainer {...toastContainerProps}>
            {toasts.map((toast) => {
                const { headerContent, toastHeaderProps } = toast
                const header = (
                    <Toast.Header {...toastHeaderProps}>
                        {headerContent}
                    </Toast.Header>
                )

                const { bodyContent, toastBodyProps } = toast
                const body = (
                    <Toast.Body {...toastBodyProps}>{bodyContent}</Toast.Body>
                )

                const { toastProps } = toast
                const { onClose } = toastProps ?? {}
                delete toastProps?.onClose
                return (
                    <Toast
                        key={toast.id}
                        {...toastProps}
                        onClose={() => {
                            setToasts((toastsState) =>
                                toastsState.filter((t) => t.id !== toast.id),
                            )
                            onClose?.()
                        }}
                    >
                        {header}
                        {body}
                    </Toast>
                )
            })}
        </ToastContainer>
    )
})

const ToastsProvider = ({
    children,
    toastContainerProps,
    limit,
}: {
    children: JSX.Element
    toastContainerProps?: ToastContainerProps
    limit?: number
}) => {
    const toastsRef = useRef<ToastsHandle | null>(null)

    return (
        <ToastsContext.Provider value={toastsRef}>
            {children}
            <Toasts
                ref={toastsRef}
                toastContainerProps={toastContainerProps}
                limit={limit}
            ></Toasts>
        </ToastsContext.Provider>
    )
}

export default ToastsProvider
