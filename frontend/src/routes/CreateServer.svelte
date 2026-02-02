<script>
	import { Steps } from '@skeletonlabs/skeleton-svelte';
	let step = $state(0);
</script>

<div class="container mx-auto">
    <h2 class="h2">Create your new Minecraft server</h2>

    <hr class="my-6" />

    <Steps {step} onStepChange={(details) => (step = details.step)} count={3} class="w-full">
    	<Steps.List>
    			<Steps.Item index={0}>
    				<Steps.Trigger>
    					<Steps.Indicator>1</Steps.Indicator>
    					Basic Information
    				</Steps.Trigger>
    				<Steps.Separator />
    			</Steps.Item>
                <Steps.Item index={1}>
    				<Steps.Trigger>
    					<Steps.Indicator>2</Steps.Indicator>
    					Allocate resources
    				</Steps.Trigger>
    				<Steps.Separator />
    			</Steps.Item>
                <Steps.Item index={2}>
    				<Steps.Trigger>
    					<Steps.Indicator>3</Steps.Indicator>
    					Advanced settings
    				</Steps.Trigger>
    			</Steps.Item>
    	</Steps.List>
    	<Steps.Content index={0}>
    		First step content goes here.
    	</Steps.Content>
        <Steps.Content index={1}>
    		Second step content goes here.
        </Steps.Content>
        <Steps.Content index={2}>
    		Third step content goes here.
    	</Steps.Content>
    	<div class="flex justify-between items-center gap-2">
    		<Steps.PrevTrigger class="btn preset-filled">Back</Steps.PrevTrigger>
    		<Steps.NextTrigger class="btn preset-filled">Next</Steps.NextTrigger>
    	</div>
    </Steps>

</div>