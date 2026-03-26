# Get all subdirectories in the current path
$directories = Get-ChildItem -Directory

foreach ($dir in $directories) {
    $dirName = $dir.Name
    # Get all PNG files in the subdirectory
    $pngFiles = Get-ChildItem -Path $dir.FullName -Filter "*.png" | Select-Object -ExpandProperty FullName

    if ($pngFiles) {
        Write-Host "Creating $dirName.ico from files in $($dir.Name)..."

        # Run ImageMagick convert command
        # This combines all PNGs into a single multi-resolution ICO file
        magick convert $pngFiles "$dirName.ico"
    }
}

Write-Host "All icons have been created in the current directory." -ForegroundColor Green