% Fast Fourier Transform Example
% Demonstrates signal processing in MATLAB

function [freq, magnitude] = analyze_signal(signal, fs)
    % ANALYZE_SIGNAL Compute frequency spectrum of a signal
    %   [freq, magnitude] = analyze_signal(signal, fs)
    %   signal - input time-domain signal
    %   fs - sampling frequency in Hz
    
    n = length(signal);
    
    % Apply Hanning window to reduce spectral leakage
    window = hanning(n);
    windowed_signal = signal .* window';
    
    % Compute FFT
    fft_result = fft(windowed_signal);
    
    % Compute single-sided spectrum
    magnitude = abs(fft_result / n);
    magnitude = magnitude(1:n/2+1);
    magnitude(2:end-1) = 2 * magnitude(2:end-1);
    
    % Frequency vector
    freq = fs * (0:(n/2)) / n;
end

% Example usage
fs = 1000;  % Sampling frequency
t = 0:1/fs:1-1/fs;  % Time vector

% Create test signal: 50 Hz + 120 Hz
signal = sin(2*pi*50*t) + 0.5*sin(2*pi*120*t);

% Add noise
noisy_signal = signal + 0.2*randn(size(t));

% Analyze
[f, mag] = analyze_signal(noisy_signal, fs);

% Plot results
figure;
subplot(2,1,1);
plot(t, noisy_signal);
xlabel('Time (s)');
ylabel('Amplitude');
title('Time Domain Signal');

subplot(2,1,2);
plot(f, mag);
xlabel('Frequency (Hz)');
ylabel('Magnitude');
title('Frequency Spectrum');
