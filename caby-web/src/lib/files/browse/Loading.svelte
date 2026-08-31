<script lang="ts">
	import IconLucideSquareCheckBig from '~icons/lucide/square-check-big';
	import IconFlatColorIconsFile from '~icons/flat-color-icons/file';
</script>

<tr>
	<td data-cell="select" class="check"><IconLucideSquareCheckBig /></td>
	<td data-cell="main" class="main">
		<div class="fx fx-cc">
			<div class="icon fx fx-cc"><IconFlatColorIconsFile /></div>
			<div class="text fx-grow">
				<div class="name">ㅤ</div>
				<div class="size">ㅤ</div>
			</div>
		</div>
	</td>
	<td data-cell="actions" class="actions">
		<div class="fx fx-ac">
			<span class="ghost"></span>
		</div>
	</td>
	<td data-cell="last-modified">ㅤ</td>
</tr>

<style lang="scss">
	@use 'entry';

	.actions span.ghost {
		height: 10px;
		width: 40%;
		border-radius: 5px;
		background: lightgrey;
	}
</style>
