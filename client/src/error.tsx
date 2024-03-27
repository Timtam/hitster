import Container from "react-bootstrap/Container"

export default function Error({ text }: { text: string | undefined }) {
    return (
        <Container aria-live="polite">
            <p>{text !== undefined ? `An error occurred: ${text}` : ""}</p>
        </Container>
    )
}
