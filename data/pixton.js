function pixton_anim(id, offset, duration) {
    $(id).animate({top: offset, opacity: 'toggle'}, {duration: duration, easing: 'swing'});
}

function pixton_ready() {
    pixton_anim('#upload-div', '-=2em', 500);
    setTimeout(function() {
        $('#upload-size-note').animate({opacity: 'toggle'}, 250);
    }, 500);
    var oldValue = '';
    $('#selected-file').on('change', function() {
        var label = '';
        if (this.value != '') {
            label = this.value.split(/(\\|\/)/g).pop();
            let f = $('#selected-file');
            let size = f.get(0).files[0].size;
            let isImage = (f.get(0).files[0].type.indexOf('image/') > -1);
            let sz = (size / (1024.0 * 1024.0)) + '';
            let sep = sz.indexOf('.');
            var size_label = sz;
            if ((sep + 2) < sz.length) {
                size_label = sz.substring(0, sep + 2); 
            }

            if (size > (2 * 1024 * 1024) || !isImage) {  
                if (isImage)
                    alert('File ' + label + ' is ' + size_label + ' MB big, maximum allowed size is 2MB');
                else
                    alert('File ' + label + ' is not an image: type ' + f.get(0).files[0].type);

                this.value = '';
                if (oldValue != '') {
                    pixton_anim('#button', '+=2em', 50);
                }
                oldValue = '';
                $('#upload-size-note').text('▸ Maximum upload size is 2.0MB');
                $('#file-label').text('');
                return false;
            }

            if (oldValue == '') {
                pixton_anim('#button', '-=2em', 150);
                pixton_anim('#upload-size-note', '-=2em', 50);
                $('#upload-size-note').html('▸ File size ' + size_label + ' MB <span class=\'green\'>✔</span>');
                pixton_anim('#upload-size-note', '+=2em', 250);
            }                
        } else {
            pixton_anim('#button', '+=2em', 50);
            label = '';
            $('#upload-size-note').text('▸ Maximum upload size is 2.0MB');
        }
        $('#file-label').text(label);
        oldValue = this.value;
    });
}