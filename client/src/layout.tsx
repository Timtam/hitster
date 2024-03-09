import "primereact/resources/themes/lara-light-indigo/theme.css" //theme
import "primereact/resources/primereact.min.css" //core css
import "primeicons/primeicons.css" //icons
import "primeflex/primeflex.css" // flex
import { Outlet } from "react-router-dom"

export default function Layout() {
    return (
        <>
            <Outlet />
        </>
    )
}
