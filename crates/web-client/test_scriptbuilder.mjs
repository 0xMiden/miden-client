import { ScriptBuilder } from './dist/index.js';

console.log('Testing ScriptBuilder...');

try {
  console.log('ScriptBuilder:', typeof ScriptBuilder);
  
  if (typeof ScriptBuilder === 'function') {
    console.log('✓ ScriptBuilder is available as a constructor');
    
    // Try to create an instance
    const scriptBuilder = new ScriptBuilder(true);
    console.log('✓ ScriptBuilder instance created successfully');
    console.log('ScriptBuilder instance:', scriptBuilder);
    
    // Try to call a method
    console.log('Available methods:');
    console.log('- linkModule:', typeof scriptBuilder.linkModule);
    console.log('- linkStaticLibrary:', typeof scriptBuilder.linkStaticLibrary);
    console.log('- linkDynamicLibrary:', typeof scriptBuilder.linkDynamicLibrary);
    console.log('- compileTxScript:', typeof scriptBuilder.compileTxScript);
    console.log('- compileNoteScript:', typeof scriptBuilder.compileNoteScript);
    
  } else {
    console.log('✗ ScriptBuilder is not a constructor, type:', typeof ScriptBuilder);
  }
  
} catch (error) {
  console.error('✗ Error testing ScriptBuilder:', error.message);
}