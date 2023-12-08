<header>
    <h2>CABY</h2>
    <button>Flip Color</button>
</header>
<div>
    <slot></slot>
</div>

<style lang="scss">
    @import "../styles.css";

    header {
        border-bottom: 1px solid var(--clr-accent);
        padding: 1rem;
        display: flex;

        h2 {
            color: var(--clr-secondary);
            flex-grow: 1;
        }
    }
</style>