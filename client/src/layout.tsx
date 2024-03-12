import "bootstrap/dist/css/bootstrap.min.css"
import { Outlet } from "react-router-dom"
import Navigation from "./navigation"

export default function Layout() {
    return (
        <>
            <Navigation />
            <Outlet />
        </>
    )
}
