import { forwardRef, ReactNode, useState } from "react"

interface FocusAnchorProps {
    children: ReactNode
}

export default forwardRef<HTMLElement, FocusAnchorProps>(
    ({ children }: FocusAnchorProps, ref) => {
        let [opened, setOpened] = useState(false)

        return (
            <div
                tabIndex={-1}
                ref={(e) => {
                    if (typeof ref === "function") ref(e)
                    else if (ref) ref.current = e

                    if (e && !opened) {
                        e.focus()
                        setOpened(true)
                    }
                }}
            >
                {children}
            </div>
        )
    },
)
