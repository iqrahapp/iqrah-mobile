/**
 * Test script for wizard flow - tests all API endpoints used by the wizard
 */

const API = 'http://localhost:8000';

async function testEndpoint(name, url, expectedFields) {
  try {
    const response = await fetch(url);
    const data = await response.json();

    if (!response.ok) {
      console.error(`❌ ${name}: HTTP ${response.status}`);
      return false;
    }

    // Check if it's an array
    if (Array.isArray(data)) {
      if (data.length === 0) {
        console.warn(`⚠️  ${name}: Empty array returned`);
        return true;
      }

      // Check first item has expected fields
      const item = data[0];
      const missingFields = expectedFields.filter(f => !(f in item));

      if (missingFields.length > 0) {
        console.error(`❌ ${name}: Missing fields: ${missingFields.join(', ')}`);
        console.log('   Got:', Object.keys(item));
        return false;
      }
    }

    console.log(`✅ ${name}: OK (${Array.isArray(data) ? data.length + ' items' : 'object'})`);
    return true;

  } catch (err) {
    console.error(`❌ ${name}: ${err.message}`);
    return false;
  }
}

async function runTests() {
  console.log('🧪 Testing Tajweed Annotation Wizard API Endpoints\n');

  const tests = [
    {
      name: 'Health Check',
      url: `${API}/health`,
      fields: []
    },
    {
      name: 'Get Surahs',
      url: `${API}/api/qpc/surahs`,
      fields: ['surah', 'name_arabic', 'name_english', 'ayah_count', 'word_count']
    },
    {
      name: 'Get Ayahs (Surah 1)',
      url: `${API}/api/qpc/ayahs/1?from_ayah=1&to_ayah=3`,
      fields: ['surah', 'ayah', 'text', 'rules']
    },
    {
      name: 'Get Words (Surah 1)',
      url: `${API}/api/qpc/words?surah=1&limit=10`,
      fields: ['id', 'location', 'surah', 'ayah', 'word', 'text', 'rules']
    },
    {
      name: 'Get Taxonomy (Rules & Anti-patterns)',
      url: `${API}/api/taxonomy/`,
      fields: []  // Different structure, check separately
    },
    {
      name: 'Get Available Rules',
      url: `${API}/api/qpc/rules`,
      fields: []  // Array of strings
    }
  ];

  let passed = 0;
  let failed = 0;

  for (const test of tests) {
    const result = await testEndpoint(test.name, test.url, test.fields);
    if (result) passed++;
    else failed++;
  }

  // Special test for taxonomy structure
  console.log('\n🔍 Testing Taxonomy Structure...');
  try {
    const response = await fetch(`${API}/api/taxonomy/`);
    const data = await response.json();

    if (data.rules && data.anti_patterns) {
      console.log(`✅ Taxonomy has rules and anti_patterns`);
      console.log(`   Rules: ${data.rules.length}`);
      console.log(`   Anti-pattern categories: ${Object.keys(data.anti_patterns).length}`);

      // Count total anti-patterns
      let totalAP = 0;
      for (const category in data.anti_patterns) {
        totalAP += data.anti_patterns[category].length;
      }
      console.log(`   Total anti-patterns: ${totalAP}`);
      passed++;
    } else {
      console.error(`❌ Taxonomy structure invalid`);
      failed++;
    }
  } catch (err) {
    console.error(`❌ Taxonomy test failed: ${err.message}`);
    failed++;
  }

  console.log('\n' + '='.repeat(50));
  console.log(`📊 Results: ${passed} passed, ${failed} failed`);
  console.log('='.repeat(50));

  if (failed === 0) {
    console.log('\n✨ All API endpoints working correctly!');
    console.log('\n📝 Manual Testing Checklist:');
    console.log('   1. Open http://localhost:5174 in browser');
    console.log('   2. Click "Annotation Wizard"');
    console.log('   3. Stage 0: Select Surah 1, Ayahs 1-3');
    console.log('   4. Stage 1: Record or upload audio, verify auto-trim');
    console.log('   5. Stage 2: Segment verses (3 segments)');
    console.log('   6. Stage 3: Segment words for each ayah');
    console.log('   7. Stage 4: Add anti-pattern annotations');
    console.log('   8. Export and verify JSON structure');
    console.log('   9. Test Load feature with exported JSON');
    return 0;
  } else {
    console.log('\n⚠️  Some tests failed. Check backend configuration.');
    return 1;
  }
}

runTests().then(code => process.exit(code));
