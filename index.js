async function doSomething() {
    console.log("Starting an interesting async operation...");
    
    // Simulate some async work
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    console.log("Halfway there...");
    
    // More simulated async work
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    console.log("Operation complete!");
    
    return "Interesting result";
}
