import type { HTMLAttributes, JSX } from "react"
import type { ToastHeaderProps, ToastProps } from "react-bootstrap"

import type { BsPrefixProps } from "react-bootstrap/helpers"

export type ToastIdType = number

export type ToastPropsOmitBg = Omit<ToastProps, "bg">

export type ToastOptions<T extends ToastProps> = {
    headerContent: string | JSX.Element
    bodyContent: string | JSX.Element
    toastProps?: T
    toastHeaderProps?: ToastHeaderProps
    toastBodyProps?: BsPrefixProps & HTMLAttributes<HTMLElement>
}

export type ToastOptionsWithId = ToastOptions<ToastProps> & { id: ToastIdType }

export interface ToastsHandle {
    show: (toastOptionsWithId: ToastOptionsWithId) => void
    hide: (id: ToastIdType) => void
}
