# Voice Biometrics Frontend Implementation Plan

**Epic:** Voice Biometrics (Speaker Recognition)
**Phase:** AC2 - Voice Enrollment UI
**Date:** 2025-10-11
**Status:** Planning Phase

---

## Executive Summary

This document provides a comprehensive plan for implementing the frontend enrollment UI for voice biometrics. The design prioritizes **trust, clarity, and ease of use** - critical for a feature that collects sensitive biometric data.

**Key Principle:** Voice enrollment is a sensitive operation that requires explicit user consent and clear feedback at every step. The UI must be intuitive enough for non-technical users while providing sufficient information for power users.

---

## 1. User Experience Goals

### 1.1 Primary Goals

1. **Build Trust**
   - Clear explanation of what voice biometrics does
   - Transparent about data storage (local only)
   - Explicit consent before enrollment

2. **Minimize Friction**
   - Simple 3-sample enrollment process
   - Clear audio quality feedback
   - One-click re-recording if needed

3. **Provide Confidence**
   - Real-time feedback during recording
   - Quality indicators after each sample
   - Success confirmation with stats

4. **Enable Management**
   - List all enrolled users
   - Easy profile deletion
   - Clear active/inactive status

### 1.2 Non-Goals (Future Enhancements)

- ❌ Real-time recognition feedback during conversations (AC3)
- ❌ Per-user personalization settings (AC4)
- ❌ Voice print export/import
- ❌ Multi-device synchronization

---

## 2. UI Component Hierarchy

```
SettingsModal
  └─ Tabs
      ├─ General
      ├─ Voice
      ├─ Integrations
      │   ├─ Spotify
      │   ├─ Home Assistant
      │   └─ Devices
      └─ 🆕 User Profiles (NEW)  ← Voice Biometrics
          ├─ UserProfilesSettings
          │   ├─ ProfileList
          │   │   └─ ProfileCard (for each user)
          │   └─ AddUserButton
          └─ EnrollmentModal (overlay)
              ├─ WelcomeStep
              ├─ NameInputStep
              ├─ RecordingSt

ep (×3)
              ├─ ProcessingStep
              └─ SuccessStep
```

---

## 3. Component Specifications

### 3.1 UserProfilesSettings Component

**Purpose:** Main settings panel for managing voice profiles

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  User Profiles                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📊 2 users enrolled                                        │
│  🔒 All voice data stored locally and securely             │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  👤 John Doe                                          │ │
│  │  ✅ Active • Enrolled 3 days ago                      │ │
│  │  📈 Recognized 47 times                               │ │
│  │                                      [Delete Profile] │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  👤 Jane Smith                                        │ │
│  │  ✅ Active • Enrolled 1 week ago                      │ │
│  │  📈 Recognized 23 times                               │ │
│  │                                      [Delete Profile] │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌─────────────────────────────────────────┐              │
│  │  ➕ Add New User                        │              │
│  └─────────────────────────────────────────┘              │
│                                                             │
│  💡 Tip: Aura uses your voice to provide personalized     │
│     responses and access your specific data (like your     │
│     Spotify playlists or calendar).                        │
└─────────────────────────────────────────────────────────────┘
```

**State Management:**
```typescript
interface UserProfilesState {
  profiles: UserProfile[];
  isLoading: boolean;
  error: string | null;
  showEnrollmentModal: boolean;
}

interface UserProfile {
  id: number;
  name: string;
  enrollmentDate: string;
  lastRecognized: string | null;
  recognitionCount: number;
  isActive: boolean;
}
```

**Key Features:**
- List all enrolled users with statistics
- Delete confirmation dialog
- Loading states for async operations
- Error handling with toast notifications

### 3.2 EnrollmentModal Component

**Purpose:** Multi-step wizard for voice enrollment

**Modal Overlay:** Full-screen modal with escape to cancel

#### Step 1: Welcome Screen

```
┌─────────────────────────────────────────────────────────────┐
│                                                         [X] │
│                       Voice Enrollment                      │
│                                                             │
│                           🎤                                │
│                                                             │
│  Let Aura recognize your voice for personalized responses  │
│                                                             │
│  How it works:                                              │
│  1. You'll speak 3 short phrases                           │
│  2. Aura creates a secure "voice print"                    │
│  3. Your voice print is stored only on this device         │
│                                                             │
│  📊 Success rate: ~95% accuracy                            │
│  ⏱️  Takes about 2 minutes                                 │
│  🔒 100% private (no cloud upload)                         │
│                                                             │
│                          [Get Started]                      │
│                                                             │
│  By continuing, you consent to Aura collecting and storing │
│  a biometric voice print on this device for speaker        │
│  identification. Learn more about privacy →                │
└─────────────────────────────────────────────────────────────┘
```

**Key Elements:**
- Clear value proposition
- Privacy reassurance
- Time estimate (2 minutes)
- Explicit consent language
- Link to privacy policy/documentation

#### Step 2: Name Input

```
┌─────────────────────────────────────────────────────────────┐
│                                                         [X] │
│                       Voice Enrollment                      │
│                      Step 1 of 5: Your Name                 │
│                                                             │
│                                                             │
│  👤 What should Aura call you?                              │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Enter your name                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  💡 This name will appear when Aura recognizes your voice  │
│                                                             │
│                                                             │
│                [Back]                         [Continue →] │
└─────────────────────────────────────────────────────────────┘
```

**Validation:**
- Required field
- 2-50 characters
- No duplicate names
- Unicode support (international names)

#### Step 3-5: Recording Samples (×3)

```
┌─────────────────────────────────────────────────────────────┐
│                                                         [X] │
│                       Voice Enrollment                      │
│                   Step 2 of 5: Voice Sample 1               │
│                                                             │
│  🎤 Please say:                                             │
│                                                             │
│     "Hey Aura, this is [Your Name]"                         │
│                                                             │
│                                                             │
│                  ┌───────────────────┐                      │
│                  │                   │                      │
│                  │   🔴 Recording    │  ← Pulsing animation │
│                  │                   │                      │
│                  │  ▓▓▓▓▓▓▓░░░░░░░  │  ← Audio level meter │
│                  │                   │                      │
│                  └───────────────────┘                      │
│                                                             │
│                     [Stop Recording]                        │
│                                                             │
│  💡 Speak clearly in a normal voice                         │
│  🔊 Make sure you're in a quiet environment                 │
│                                                             │
│                [Back]                        [Skip Sample] │
└─────────────────────────────────────────────────────────────┘
```

**After Recording (Quality Check):**
```
┌─────────────────────────────────────────────────────────────┐
│                       Voice Enrollment                      │
│                   Step 2 of 5: Voice Sample 1               │
│                                                             │
│  ✅ Sample recorded successfully!                           │
│                                                             │
│  Quality: ████████░░ Excellent (8/10)                       │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  🔊 [▶️ Play Back]                                    │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│                                                             │
│            [Re-record]              [Use This Sample →]    │
└─────────────────────────────────────────────────────────────┘
```

**Recommended Phrases:**
1. "Hey Aura, this is [Name]"
2. "My name is [Name]"
3. "I use Aura for my smart home"

**Recording States:**
- ⏺️  Ready to record
- 🔴 Recording (0-10 seconds)
- ✅ Sample captured
- ⚠️ Quality too low (prompt re-record)
- ❌ Error (microphone issue)

**Audio Quality Indicators:**
- Volume level (real-time waveform)
- Duration (minimum 2 seconds, maximum 10 seconds)
- Silence detection (warn if too quiet)
- Background noise detection (warn if too noisy)

#### Step 6: Processing

```
┌─────────────────────────────────────────────────────────────┐
│                       Voice Enrollment                      │
│                    Step 5 of 5: Processing                  │
│                                                             │
│                                                             │
│                     ⏳ Creating voice print...               │
│                                                             │
│                  ┌─────────────────────────┐                │
│                  │  ████████████████░░░░░  │  75%           │
│                  └─────────────────────────┘                │
│                                                             │
│  Analyzing voice samples...                                 │
│  ✅ Sample 1 processed                                      │
│  ✅ Sample 2 processed                                      │
│  ⏳ Processing sample 3...                                  │
│                                                             │
│  This may take a few seconds.                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### Step 7: Success

```
┌─────────────────────────────────────────────────────────────┐
│                                                         [X] │
│                       Voice Enrollment                      │
│                                                             │
│                                                             │
│                           ✅                                │
│                                                             │
│              Enrollment Complete, [Name]!                   │
│                                                             │
│  Your voice profile has been created successfully.          │
│  Aura will now recognize you automatically.                 │
│                                                             │
│  📊 Enrollment Quality: Excellent                           │
│  🎯 Expected Accuracy: ~95%                                 │
│  🔒 Stored securely on this device                          │
│                                                             │
│  💡 Next time you speak to Aura, your personalized         │
│     settings will be activated automatically.               │
│                                                             │
│                          [Done]                             │
└─────────────────────────────────────────────────────────────┘
```

**Success Metrics Displayed:**
- Enrollment quality score
- Estimated recognition accuracy
- Sample variance (hidden from user, logged for debugging)

### 3.3 ProfileCard Component

**Purpose:** Display individual user profile with stats

```typescript
interface ProfileCardProps {
  profile: UserProfile;
  onDelete: (id: number) => Promise<void>;
}
```

**Features:**
- User name and avatar (emoji/initials)
- Enrollment date (relative time)
- Last recognized (relative time)
- Recognition count
- Delete button with confirmation

---

## 4. Audio Recording Flow

### 4.1 Browser Audio API

**Implementation:**
```typescript
async function recordAudioSample(
  duration: number = 5000  // 5 seconds
): Promise<Float32Array> {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  const context = new AudioContext({ sampleRate: 16000 });  // 16kHz for model
  const source = context.createMediaStreamSource(stream);

  const processor = context.createScriptProcessor(4096, 1, 1);
  const audioData: number[] = [];

  processor.onaudioprocess = (e) => {
    const inputData = e.inputBuffer.getChannelData(0);
    audioData.push(...inputData);
  };

  source.connect(processor);
  processor.connect(context.destination);

  // Record for specified duration
  await new Promise(resolve => setTimeout(resolve, duration));

  // Stop recording
  stream.getTracks().forEach(track => track.stop());
  processor.disconnect();
  source.disconnect();

  return new Float32Array(audioData);
}
```

**Key Considerations:**
- Request microphone permission on first use
- Handle permission denial gracefully
- Real-time audio level visualization
- Stop/restart recording if needed

### 4.2 Sending Samples to Backend

```typescript
async function enrollUser(
  name: string,
  samples: Float32Array[]
): Promise<number> {
  const result = await invoke<number>("biometrics_enroll_user", {
    userName: name,
    audioSamples: samples.map(s => Array.from(s))
  });

  return result;  // User ID
}
```

---

## 5. Error Handling & Edge Cases

### 5.1 Error Scenarios

| Error | User-Facing Message | Recovery Action |
|-------|---------------------|-----------------|
| Microphone permission denied | "Aura needs microphone access to enroll your voice. Please enable it in your browser settings." | Link to browser help |
| Insufficient samples | "Please record at least 3 voice samples." | Retry recording |
| Poor audio quality | "Audio quality is too low. Please try again in a quieter environment." | Re-record sample |
| Inconsistent samples | "Your voice samples don't match closely enough. This might happen if there's background noise. Let's try again." | Restart enrollment |
| Duplicate user name | "A user with this name already exists. Please choose a different name." | Edit name field |
| Database error | "Failed to save voice profile. Please try again." | Retry button |
| No microphone detected | "No microphone detected. Please connect a microphone and try again." | Check hardware |

### 5.2 Edge Case Handling

**User cancels mid-enrollment:**
- Prompt: "Are you sure you want to cancel? Your progress will be lost."
- Action: Discard partial data, return to profile list

**App closes during enrollment:**
- No partial data saved (atomic operation)
- User can restart enrollment cleanly

**User clicks "Back" during recording:**
- Stop recording immediately
- Discard incomplete sample
- Return to previous step

**Network interruption (N/A for this feature):**
- Not applicable (all processing is local)

---

## 6. Accessibility Considerations

### 6.1 WCAG 2.1 AA Compliance

✅ **Keyboard Navigation**
- All interactive elements accessible via Tab/Shift+Tab
- Enter to confirm, Escape to cancel
- Focus indicators clearly visible

✅ **Screen Reader Support**
- ARIA labels for all UI elements
- Live regions for status updates
- Clear heading hierarchy

✅ **Color Contrast**
- 4.5:1 contrast ratio for normal text
- 3:1 for large text and UI components
- Not relying on color alone for information

✅ **Motion & Animation**
- Respect `prefers-reduced-motion` setting
- Disable pulsing animations if requested
- Provide static alternatives

### 6.2 Internationalization (i18n)

**Supported Languages (Future):**
- English (en-US) - Initial release
- Spanish (es-ES)
- French (fr-FR)
- German (de-DE)
- Japanese (ja-JP)

**Translation Keys:**
```typescript
const i18n = {
  "enrollment.welcome.title": "Voice Enrollment",
  "enrollment.welcome.description": "Let Aura recognize your voice...",
  "enrollment.step.recording": "Step {current} of {total}: Voice Sample {sample}",
  // ... etc
}
```

---

## 7. Implementation Checklist

### 7.1 Backend (Tauri Commands)

```rust
// In lib.rs

/// State for voice biometrics engine
pub type VoiceBiometricsState = Arc<Mutex<VoiceBiometrics>>;

#[tauri::command]
async fn biometrics_enroll_user(
    user_name: String,
    audio_samples: Vec<Vec<f32>>,
    biometrics: State<'_, VoiceBiometricsState>,
) -> Result<i64, String> {
    biometrics.lock().await
        .enroll_user(user_name, audio_samples)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn biometrics_list_users(
    biometrics: State<'_, VoiceBiometricsState>,
) -> Result<Vec<UserProfile>, String> {
    biometrics.lock().await
        .list_all_users()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn biometrics_delete_user(
    user_id: i64,
    biometrics: State<'_, VoiceBiometricsState>,
) -> Result<(), String> {
    biometrics.lock().await
        .delete_user_profile(user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn biometrics_get_status(
    biometrics: State<'_, VoiceBiometricsState>,
) -> Result<BiometricsStatus, String> {
    let users = biometrics.lock().await
        .list_all_users()
        .await
        .map_err(|e| e.to_string())?;

    Ok(BiometricsStatus {
        enrolled_user_count: users.len(),
        is_enabled: true,
    })
}

// In run() function setup:
let biometrics_engine = VoiceBiometrics::new(database.clone());
app.manage(Arc::new(Mutex::new(biometrics_engine)));

// Register commands:
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    biometrics_enroll_user,
    biometrics_list_users,
    biometrics_delete_user,
    biometrics_get_status,
])
```

### 7.2 Frontend Components

**Files to create:**
1. `src/components/UserProfilesSettings.tsx` - Main settings panel
2. `src/components/EnrollmentModal.tsx` - Enrollment wizard
3. `src/components/ProfileCard.tsx` - User profile display
4. `src/hooks/useAudioRecording.ts` - Audio capture hook
5. `src/hooks/useBiometrics.ts` - Biometrics state management

**Files to modify:**
1. `src/components/SettingsModal.tsx` - Add "User Profiles" tab
2. `src/utils/audio.ts` - Add audio processing utilities (if needed)

### 7.3 Step-by-Step Implementation Order

**Session 1: Backend Commands (1 hour)**
- [ ] Add Tauri commands to lib.rs
- [ ] Initialize VoiceBiometrics in app setup
- [ ] Register commands in invoke_handler
- [ ] Test commands with curl/Postman

**Session 2: Audio Recording Hook (45 minutes)**
- [ ] Create `useAudioRecording` hook
- [ ] Implement microphone permission handling
- [ ] Add real-time audio level visualization
- [ ] Test in isolated component

**Session 3: Enrollment Modal (1.5 hours)**
- [ ] Create EnrollmentModal component
- [ ] Implement multi-step wizard logic
- [ ] Add recording UI for each step
- [ ] Integrate with backend commands
- [ ] Add error handling and validation

**Session 4: Profile Management (1 hour)**
- [ ] Create UserProfilesSettings component
- [ ] Create ProfileCard component
- [ ] Implement list/delete functionality
- [ ] Add loading and error states

**Session 5: Integration & Polish (45 minutes)**
- [ ] Add "User Profiles" tab to SettingsModal
- [ ] Ensure proper state management
- [ ] Add toast notifications
- [ ] Test end-to-end flow
- [ ] Accessibility review

**Total Estimated Time:** 5-6 hours

---

## 8. Testing Plan

### 8.1 Unit Tests

**Backend:**
- ✅ Already complete (voice_biometrics.rs tests)

**Frontend:**
```typescript
// Example test for EnrollmentModal
describe("EnrollmentModal", () => {
  it("should render welcome step initially", () => {
    render(<EnrollmentModal isOpen={true} />);
    expect(screen.getByText("Voice Enrollment")).toBeInTheDocument();
  });

  it("should validate name input", () => {
    // Test empty name rejection
    // Test duplicate name rejection
  });

  it("should require 3 samples", () => {
    // Test that Continue button is disabled until 3 samples recorded
  });
});
```

### 8.2 Integration Tests

**Enrollment Flow:**
1. Open Settings → User Profiles
2. Click "Add New User"
3. Complete enrollment wizard with 3 samples
4. Verify user appears in list
5. Verify database has new profile

**Delete Flow:**
1. Click "Delete Profile" on existing user
2. Confirm deletion
3. Verify user removed from list
4. Verify database record deleted

### 8.3 Manual Testing Checklist

- [ ] Enrollment works with clear audio
- [ ] Enrollment rejects noisy audio
- [ ] Re-recording samples works
- [ ] Canceling enrollment discards data
- [ ] Multiple users can be enrolled
- [ ] Deleting user removes from list
- [ ] UI is responsive and smooth
- [ ] Error messages are clear
- [ ] Accessibility (keyboard nav, screen reader)
- [ ] Cross-platform (Windows, macOS, Linux)

---

## 9. Future Enhancements (Post-AC2)

### 9.1 Phase 2 Features (AC3)

- Real-time speaker identification during conversations
- Visual indicator when user is recognized
- Voice-driven profile switching

### 9.2 Phase 3 Features (AC4)

- Per-user personalization settings
- User-specific conversation history
- Spotify playlist personalization
- Home Assistant scene personalization

### 9.3 Advanced Features (v2.0)

- Voice print quality score visualization
- Re-enrollment wizard (improve accuracy)
- Profile export/import (backup/restore)
- Family mode (parental controls)
- Guest mode (temporary profiles)

---

## 10. Privacy & Consent UX

### 10.1 Consent Flow

**First-time enrollment:**
1. Show privacy explanation modal
2. Explain data collection and storage
3. Link to full privacy policy
4. Require explicit "I Consent" button click
5. Store consent timestamp in database

**Subsequent enrollments:**
- Skip consent modal (already granted)
- Show brief reminder: "Voice data stored locally only"

### 10.2 Privacy Information Display

**In UserProfilesSettings:**
```
🔒 Your Privacy
- All voice data is stored locally on this device
- Voice prints never leave your computer
- You can delete your profile at any time
- [Learn more about voice biometrics →]
```

**Link to documentation:**
- Detailed privacy policy
- How voice biometrics works
- Security measures
- Data retention policy
- User rights (access, deletion, portability)

---

## 11. Wireframe Summary

### 11.1 Main Screens

1. **User Profiles Settings** - List view with stats
2. **Enrollment Modal** - 7-step wizard
3. **Delete Confirmation** - Simple dialog

### 11.2 Key User Flows

**Happy Path:**
1. Open Settings → User Profiles
2. Click "Add New User"
3. Review welcome screen → Continue
4. Enter name → Continue
5. Record 3 samples (each: record → review → accept)
6. Wait for processing
7. See success screen
8. Return to profile list (new user visible)

**Error Recovery:**
1. Poor quality sample → Re-record
2. Inconsistent samples → Restart enrollment
3. Duplicate name → Edit name and retry

---

## 12. Success Criteria (AC2)

✅ **Functional Requirements:**
- [ ] User can enroll with 3-5 voice samples
- [ ] User receives quality feedback on each sample
- [ ] Voice print is stored securely in database
- [ ] User can view list of enrolled profiles
- [ ] User can delete profiles
- [ ] Enrollment wizard is intuitive and clear

✅ **Non-Functional Requirements:**
- [ ] Enrollment completes in <2 minutes
- [ ] UI is responsive (no blocking operations)
- [ ] Error messages are actionable
- [ ] Accessible (keyboard + screen reader)
- [ ] Works on Windows, macOS, Linux

✅ **UX Requirements:**
- [ ] User understands what voice biometrics does
- [ ] User feels confident their data is private
- [ ] User knows how to re-enroll if needed
- [ ] Visual feedback is clear and immediate

---

## Appendix: Design Tokens

### Color Scheme (Dark Mode)

```css
--color-bg-primary: #1a1a1a;
--color-bg-secondary: #2a2a2a;
--color-bg-tertiary: #3a3a3a;

--color-text-primary: #e0e0e0;
--color-text-secondary: #a0a0a0;
--color-text-tertiary: #707070;

--color-accent-primary: #4a90e2;    /* Blue */
--color-accent-success: #5cb85c;    /* Green */
--color-accent-warning: #f0ad4e;    /* Yellow */
--color-accent-danger: #d9534f;     /* Red */

--color-biometrics-primary: #9b59b6;  /* Purple (trust/security) */
--color-biometrics-secondary: #8e44ad;
```

### Typography

```css
--font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
--font-size-sm: 12px;
--font-size-base: 14px;
--font-size-lg: 16px;
--font-size-xl: 20px;
--font-size-2xl: 24px;
```

### Spacing

```css
--spacing-xs: 4px;
--spacing-sm: 8px;
--spacing-md: 16px;
--spacing-lg: 24px;
--spacing-xl: 32px;
```

---

**Document Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Ready for Implementation
