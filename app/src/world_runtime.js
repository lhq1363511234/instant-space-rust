(function () {
  'use strict';

  if (window.__inspaceWorldRuntimeInstalled) return;
  window.__inspaceWorldRuntimeInstalled = true;

  var PHASER_SRC = '/inspace/vendor/phaser/phaser-3.90.0-arcade.min.js';
  var HOME_LAYOUT = '/inspace/vendor/world/cloud-home-courtyard.json';
  var phaserPromise = null;
  var layoutPromise = null;
  var instances = new Map();

  function loadScript(src) {
    return new Promise(function (resolve, reject) {
      var existing = document.querySelector('script[data-inspace-world-engine]');
      if (existing) {
        if (window.Phaser) return resolve(window.Phaser);
        existing.addEventListener('load', function () { resolve(window.Phaser); }, { once: true });
        existing.addEventListener('error', reject, { once: true });
        return;
      }
      var script = document.createElement('script');
      script.src = src;
      script.async = true;
      script.dataset.inspaceWorldEngine = 'phaser-3.90.0';
      script.onload = function () { resolve(window.Phaser); };
      script.onerror = function () { reject(new Error('Phaser runtime failed to load')); };
      document.head.appendChild(script);
    });
  }

  function ensurePhaser() {
    if (window.Phaser) return Promise.resolve(window.Phaser);
    if (!phaserPromise) phaserPromise = loadScript(PHASER_SRC);
    return phaserPromise;
  }

  function ensureHomeLayout() {
    if (!layoutPromise) {
      layoutPromise = fetch(HOME_LAYOUT, { credentials: 'same-origin' })
        .then(function (response) {
          if (!response.ok) throw new Error('Cloud-home layout failed to load');
          return response.json();
        })
        .catch(function () { return null; });
    }
    return layoutPromise;
  }

  function parsePayload(host) {
    try { return JSON.parse(host.dataset.worldPayload || '{}'); }
    catch (error) { throw new Error('Invalid world payload'); }
  }

  function text(payload, zh, en) {
    return payload.locale === 'zh' ? zh : en;
  }

  function localName(payload, object) {
    return payload.locale === 'zh' || !object.name_en ? object.name_zh : object.name_en;
  }

  function stateController(root, payload) {
    var status = root.querySelector('[data-world-status]');
    var prompt = root.querySelector('[data-world-action]');
    var labels = {
      Loading: ['正在推门入内', 'Opening the Space'],
      Spawn: ['已在门前落脚', 'Arrived at the gate'],
      Idle: ['点地面行走', 'Tap the ground to walk'],
      Moving: ['正在走近', 'Walking closer'],
      NearObject: ['已走到近前', 'Close enough to interact'],
      ActionPrompt: ['可与眼前之物互动', 'An action is available'],
      ObjectOpen: ['正在查看', 'Viewing'],
      PortalConfirm: ['传送门已开启', 'Portal ready'],
      Teleporting: ['正在前往下一处空间', 'Travelling to another Space'],
      Arrive: ['已抵达', 'Arrived'],
      Error: ['场景暂时无法运行', 'The scene cannot run'],
      Offline: ['网络已断开，仍可在院中行走', 'Offline — the courtyard remains available']
    };
    var current = 'Loading';
    function set(next, detail) {
      current = next;
      root.dataset.worldState = next;
      if (status) {
        var pair = labels[next] || labels.Idle;
        status.textContent = detail || text(payload, pair[0], pair[1]);
      }
    }
    function showPrompt(object, handler) {
      if (!prompt) return;
      prompt.hidden = false;
      prompt.textContent = text(payload, '查看 ', 'Open ') + localName(payload, object);
      prompt.onclick = handler;
      set('ActionPrompt');
    }
    function hidePrompt() {
      if (!prompt) return;
      prompt.hidden = true;
      prompt.onclick = null;
      if (current === 'ActionPrompt' || current === 'NearObject') set('Idle');
    }
    return { set: set, showPrompt: showPrompt, hidePrompt: hidePrompt, current: function () { return current; } };
  }

  function sheetController(root, payload, state) {
    var sheet = root.querySelector('[data-world-sheet]');
    var curtain = root.querySelector('[data-world-sheet-curtain]');
    var close = root.querySelector('[data-world-sheet-close]');
    var cards = Array.prototype.slice.call(root.querySelectorAll('[data-world-object-card]'));
    var lastFocus = null;

    function closeSheet() {
      if (!sheet || sheet.hidden) return;
      sheet.hidden = true;
      root.classList.remove('world-sheet-open');
      cards.forEach(function (card) { card.hidden = true; });
      state.set('Idle');
      if (lastFocus && typeof lastFocus.focus === 'function') lastFocus.focus();
    }

    function openObject(object) {
      if (!sheet) return;
      var card = cards.find(function (candidate) { return candidate.dataset.worldObjectCard === object.id; });
      if (!card) return;
      cards.forEach(function (candidate) { candidate.hidden = candidate !== card; });
      lastFocus = document.activeElement;
      sheet.hidden = false;
      root.classList.add('world-sheet-open');
      var portal = object.kind === 'portal' && object.target_space_id;
      state.set(portal ? 'PortalConfirm' : 'ObjectOpen', localName(payload, object));
      window.setTimeout(function () { if (close) close.focus(); }, 40);
    }

    if (close) close.addEventListener('click', closeSheet);
    if (curtain) curtain.addEventListener('click', closeSheet);
    root.addEventListener('keydown', function (event) {
      if (event.key === 'Escape') {
        closeSheet();
        return;
      }
      if (event.key === 'Tab' && sheet && !sheet.hidden) {
        var focusable = Array.prototype.slice.call(sheet.querySelectorAll('button:not([hidden]), a[href]:not([hidden])'))
          .filter(function (element) { return element.offsetParent !== null; });
        if (!focusable.length) return;
        var first = focusable[0];
        var last = focusable[focusable.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    });
    root.querySelectorAll('[data-world-fallback-object]').forEach(function (button) {
      button.addEventListener('click', function () {
        var object = payload.bundle.objects.find(function (item) { return item.id === button.dataset.worldFallbackObject; });
        if (object) openObject(object);
      });
    });
    return { openObject: openObject, close: closeSheet };
  }

  function objectCopy(payload, object) {
    var config = object.config || {};
    return payload.locale === 'zh' || !config.copy_en
      ? (config.copy_zh || '此物尚待主理人补上一段来历。')
      : config.copy_en;
  }

  function createWorld(host, root, payload, layout, Phaser) {
    var state = stateController(root, payload);
    var sheet = sheetController(root, payload, state);
    var reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    var lowPower = Boolean((navigator.connection && navigator.connection.saveData) || (navigator.deviceMemory && navigator.deviceMemory <= 4));
    var sceneKind = payload.bundle.scene.kind;
    var objects = payload.bundle.objects || [];
    var worldWidth = sceneKind === 'home' ? 1920 : 1760;
    var worldHeight = sceneKind === 'home' ? 1080 : 980;
    var runtime = { game: null, destroyed: false };

    function shade(hex, amount) {
      var r = Math.max(0, Math.min(255, ((hex >> 16) & 255) + amount));
      var g = Math.max(0, Math.min(255, ((hex >> 8) & 255) + amount));
      var b = Math.max(0, Math.min(255, (hex & 255) + amount));
      return (r << 16) | (g << 8) | b;
    }

    function addLabel(scene, x, y, name) {
      var label = scene.add.text(x, y, name, {
        fontFamily: '"Noto Sans SC", system-ui, sans-serif',
        fontSize: '18px',
        color: '#211f1a',
        backgroundColor: 'rgba(252,251,247,0.94)',
        padding: { x: 10, y: 6 },
        align: 'center'
      }).setOrigin(0.5, 1).setDepth(90).setAlpha(0);
      return label;
    }

    function drawTree(scene, x, y, scale, autumn) {
      var crown = autumn ? 0x79856f : 0x647566;
      var g = scene.add.graphics().setDepth(Math.floor(y));
      g.fillStyle(0x5f5547, 1).fillRect(x - 10 * scale, y - 54 * scale, 20 * scale, 74 * scale);
      g.fillStyle(crown, 1).fillCircle(x, y - 80 * scale, 55 * scale);
      g.fillStyle(shade(crown, 18), 1).fillCircle(x - 34 * scale, y - 65 * scale, 38 * scale);
      g.fillStyle(shade(crown, 10), 1).fillCircle(x + 36 * scale, y - 70 * scale, 41 * scale);
      if (autumn && !lowPower) {
        g.fillStyle(0xc6a365, 0.85);
        for (var i = 0; i < 14; i += 1) g.fillCircle(x - 45 * scale + (i * 13 % 95) * scale, y - 112 * scale + (i * 19 % 78) * scale, 3 * scale);
      }
    }

    function drawHomeEnvironment(scene) {
      scene.cameras.main.setBackgroundColor('#dce4df');
      var ground = scene.add.graphics().setDepth(-20);
      ground.fillStyle(0xdce4df, 1).fillRect(0, 0, worldWidth, worldHeight);
      ground.fillStyle(0xc3ceba, 1).fillRect(0, 300, worldWidth, worldHeight - 300);
      ground.fillStyle(0xb5c4b1, 0.42);
      for (var gx = 0; gx < worldWidth; gx += 64) {
        for (var gy = 320; gy < worldHeight; gy += 64) {
          if ((gx / 64 + gy / 64) % 3 === 0) ground.fillRect(gx + 12, gy + 18, 3, 11);
        }
      }

      var distant = scene.add.graphics().setDepth(-15);
      distant.fillStyle(0x9baaa2, 0.68).fillTriangle(0, 340, 410, 35, 760, 340);
      distant.fillStyle(0xaeb9b2, 0.78).fillTriangle(490, 340, 940, 85, 1320, 340);
      distant.fillStyle(0x8f9f96, 0.63).fillTriangle(1050, 340, 1490, 25, 1920, 340);

      var path = scene.add.graphics().setDepth(-8);
      path.fillStyle(0xd8cfbc, 1).fillPoints([
        new Phaser.Geom.Point(870, 1080), new Phaser.Geom.Point(1050, 1080),
        new Phaser.Geom.Point(1110, 610), new Phaser.Geom.Point(810, 610)
      ], true);
      path.fillStyle(0xcbbfa8, 0.75);
      for (var p = 0; p < 18; p += 1) path.fillRoundedRect(852 + (p % 2) * 76 + (p % 3) * 5, 1010 - p * 23, 74, 18, 7);

      var pond = scene.add.graphics().setDepth(-6);
      pond.fillStyle(0x88a7ad, 1).fillEllipse(330, 790, 520, 270);
      pond.lineStyle(12, 0xa8b69f, 1).strokeEllipse(330, 790, 530, 280);
      if (!lowPower) {
        pond.lineStyle(3, 0xe8eee8, 0.46);
        pond.strokeEllipse(320, 760, 260, 76).strokeEllipse(410, 835, 210, 58);
      }

      var house = scene.add.graphics().setDepth(210);
      house.fillStyle(0x26342f, 1).fillTriangle(650, 185, 1270, 185, 1175, 75);
      house.fillStyle(0x35463f, 1).fillRect(650, 180, 620, 25);
      house.fillStyle(0xe9e3d5, 1).fillRect(720, 205, 480, 250);
      house.lineStyle(8, 0x6f6659, 1).strokeRect(720, 205, 480, 250);
      house.fillStyle(0x5c3e34, 1).fillRect(900, 300, 120, 155);
      house.fillStyle(0xb8c8c3, 1).fillRect(760, 260, 100, 95).fillRect(1060, 260, 100, 95);
      house.lineStyle(5, 0x6f6659, 1).strokeRect(760, 260, 100, 95).strokeRect(1060, 260, 100, 95);
      house.fillStyle(0x9f3f30, 1).fillRect(885, 215, 150, 48);

      var wall = scene.add.graphics().setDepth(40);
      wall.fillStyle(0xeee9dd, 1).fillRect(110, 340, 70, 400).fillRect(1740, 340, 70, 530);
      wall.fillStyle(0x44554c, 1).fillRect(95, 330, 100, 18).fillRect(1725, 330, 100, 18);

      var decorations = layout && layout.layers ? layout.layers.find(function (layer) { return layer.name === 'decorations'; }) : null;
      (decorations ? decorations.objects : []).forEach(function (item) {
        if (item.type === 'tree') drawTree(scene, item.x, item.y + 140, item.name === 'osmanthus' ? 1.3 : 1, item.name === 'osmanthus');
        if (item.type === 'rock') {
          var rock = scene.add.graphics().setDepth(Math.floor(item.y));
          rock.fillStyle(0x7e8782, 1).fillEllipse(item.x, item.y, 94, 54);
          rock.fillStyle(0xa8afaa, 0.65).fillEllipse(item.x - 15, item.y - 9, 44, 19);
        }
        if (item.type === 'lantern') {
          var lamp = scene.add.graphics().setDepth(Math.floor(item.y));
          lamp.fillStyle(0x4a463f, 1).fillRect(item.x - 4, item.y - 44, 8, 55);
          lamp.fillStyle(0xa43b2d, 1).fillRoundedRect(item.x - 12, item.y - 60, 24, 24, 3);
        }
      });
    }

    function drawPlaceEnvironment(scene) {
      scene.cameras.main.setBackgroundColor('#dfe6e3');
      var g = scene.add.graphics().setDepth(-20);
      g.fillStyle(0xdfe6e3, 1).fillRect(0, 0, worldWidth, worldHeight);
      g.fillStyle(0xc9d1c2, 1).fillRect(0, 270, worldWidth, worldHeight - 270);
      g.fillStyle(0xdad1be, 1).fillRoundedRect(210, 360, worldWidth - 420, 470, 28);
      g.lineStyle(3, 0xb3aa98, 0.7);
      for (var x = 250; x < worldWidth - 220; x += 96) g.lineBetween(x, 380, x, 810);
      for (var y = 400; y < 820; y += 72) g.lineBetween(230, y, worldWidth - 230, y);
      drawTree(scene, 260, 480, 1.2, false);
      drawTree(scene, worldWidth - 260, 500, 1.2, false);
    }

    function createObjectVisual(scene, object) {
      var x = worldWidth * object.x / 100;
      var y = worldHeight * object.y / 100;
      var kind = object.kind;
      var container = scene.add.container(x, y).setDepth(Math.floor(y));
      var shadow = scene.add.ellipse(0, 20, 105, 28, 0x1f211d, 0.17);
      var art = scene.add.graphics();
      var accent = 0x667568;

      if (kind === 'building' || kind === 'tourist_center') {
        art.fillStyle(0xefe9dc, 1).fillRoundedRect(-78, -80, 156, 102, 5);
        art.fillStyle(0x314239, 1).fillTriangle(-94, -72, 94, -72, 0, -142);
        art.fillStyle(0x5c3e34, 1).fillRect(-22, -34, 44, 56);
      } else if (kind === 'message_wall' || kind === 'notice_board') {
        art.fillStyle(0x6f5d49, 1).fillRect(-60, -70, 120, 80);
        art.fillStyle(0xeee7d8, 1).fillRect(-48, -58, 96, 56);
        art.fillStyle(0xa43b2d, 1).fillCircle(32, -42, 5);
        art.fillStyle(0x6f5d49, 1).fillRect(-45, 10, 10, 45).fillRect(35, 10, 10, 45);
      } else if (kind === 'host' || kind === 'ai_guide') {
        art.fillStyle(0xdbc4a9, 1).fillCircle(0, -72, 18);
        art.fillStyle(kind === 'ai_guide' ? 0x4d6d77 : 0x667568, 1).fillRoundedRect(-25, -52, 50, 73, 18);
        art.fillStyle(0x211f1a, 1).fillEllipse(0, -91, 43, 14);
      } else if (kind === 'portal') {
        accent = 0xa43b2d;
        art.lineStyle(14, accent, 1).strokeRoundedRect(-48, -105, 96, 128, 48);
        art.lineStyle(4, 0xe7d7b7, 0.85).strokeRoundedRect(-34, -91, 68, 100, 34);
      } else if (kind === 'decoration') {
        drawTree(scene, x, y + 30, 1.05, true);
        container.setVisible(false);
      } else if (kind === 'capsule') {
        art.fillStyle(0xa43b2d, 1).fillRoundedRect(-42, -54, 84, 74, 28);
        art.fillStyle(0xe8d7b6, 1).fillCircle(0, -17, 12);
      } else {
        art.fillStyle(0x667568, 1).fillRoundedRect(-52, -55, 104, 74, 7);
        art.fillStyle(0xece6d8, 1).fillRect(-34, -37, 68, 39);
        art.lineStyle(3, 0xa43b2d, 1).lineBetween(-20, -21, 20, -21);
      }

      container.add([shadow, art]);
      var label = addLabel(scene, x, y - 92, localName(payload, object));
      var hit = scene.add.zone(x, y - 35, Math.max(100, object.width * 12), Math.max(100, object.height * 8))
        .setInteractive({ useHandCursor: true }).setDepth(2000);
      return { object: object, x: x, y: y, container: container, label: label, hit: hit, accent: accent };
    }

    var sceneConfig = {
      key: 'InspaceScene',
      create: function () {
        var scene = this;
        scene.physics.world.setBounds(0, 0, worldWidth, worldHeight);
        if (sceneKind === 'home') drawHomeEnvironment(scene); else drawPlaceEnvironment(scene);

        var spawn = payload.spawn || { x: 50, y: 84 };
        var playerX = worldWidth * spawn.x / 100;
        var playerY = worldHeight * spawn.y / 100;
        var player = scene.add.container(playerX, playerY).setDepth(Math.floor(playerY));
        var playerShadow = scene.add.ellipse(0, 10, 38, 13, 0x1f211d, 0.22);
        var playerArt = scene.add.graphics();
        playerArt.fillStyle(0xdbc4a9, 1).fillCircle(0, -38, 12);
        playerArt.fillStyle(0x211f1a, 1).fillEllipse(0, -51, 34, 10);
        playerArt.fillStyle(0x4d6d77, 1).fillRoundedRect(-15, -28, 30, 45, 11);
        playerArt.fillStyle(0x2f3b38, 1).fillRect(-13, 13, 10, 18).fillRect(3, 13, 10, 18);
        player.add([playerShadow, playerArt]);

        var objectViews = objects.map(function (object) { return createObjectVisual(scene, object); });
        var destination = new Phaser.Math.Vector2(playerX, playerY);
        var movingByPointer = false;
        var near = null;
        var pending = null;
        var speed = lowPower ? 205 : 235;
        var moveVX = 0;
        var moveVY = 0;
        var lastFrameAt = performance.now();

        var pets = [];
        var petCount = Math.min(3, Number(payload.companions_moved || 0));
        for (var i = 0; i < petCount; i += 1) {
          var pet = scene.add.container(playerX - 44 - i * 32, playerY + 22 + i * 8).setDepth(Math.floor(playerY) - 1);
          var pg = scene.add.graphics();
          pg.fillStyle(i % 2 ? 0x9b765c : 0x59656e, 1).fillEllipse(0, 0, 28, 18);
          pg.fillCircle(11, -7, 9);
          pg.fillTriangle(6, -13, 9, -23, 13, -13);
          pg.fillTriangle(14, -13, 18, -22, 20, -11);
          pet.add(pg);
          pets.push(pet);
        }

        scene.cameras.main.setBounds(0, 0, worldWidth, worldHeight);
        scene.cameras.main.startFollow(player, true, reducedMotion ? 1 : 0.075, reducedMotion ? 1 : 0.075);
        scene.cameras.main.setZoom(1);

        function moveTo(x, y, objectView) {
          destination.set(Phaser.Math.Clamp(x, 38, worldWidth - 38), Phaser.Math.Clamp(y, 50, worldHeight - 38));
          pending = objectView || null;
          movingByPointer = true;
          sheet.close();
          state.hidePrompt();
          state.set('Moving');
        }

        objectViews.forEach(function (view) {
          view.hit.on('pointerdown', function (pointer) {
            pointer.event.stopPropagation();
            moveTo(view.x, Math.min(worldHeight - 50, view.y + 88), view);
          });
        });

        scene.input.on('pointerdown', function (pointer, gameObjects) {
          if (gameObjects && gameObjects.length) return;
          moveTo(pointer.worldX, pointer.worldY, null);
        });

        var keys = scene.input.keyboard ? scene.input.keyboard.addKeys('W,A,S,D,UP,DOWN,LEFT,RIGHT,ENTER,SPACE') : null;
        if (keys) {
          keys.ENTER.on('down', function () { if (near) sheet.openObject(near.object); });
          keys.SPACE.on('down', function () { if (near) sheet.openObject(near.object); });
        }

        scene.events.on('update', function (_, delta) {
          if (runtime.destroyed) return;
          var vx = 0;
          var vy = 0;
          if (keys) {
            if (keys.A.isDown || keys.LEFT.isDown) vx -= 1;
            if (keys.D.isDown || keys.RIGHT.isDown) vx += 1;
            if (keys.W.isDown || keys.UP.isDown) vy -= 1;
            if (keys.S.isDown || keys.DOWN.isDown) vy += 1;
          }
          var keyboardMoving = vx !== 0 || vy !== 0;
          moveVX = 0;
          moveVY = 0;
          var frameAt = performance.now();
          var realDelta = Math.min(50, Math.max(0, frameAt - lastFrameAt));
          lastFrameAt = frameAt;
          var step = speed * realDelta / 1000;
          if (keyboardMoving) {
            movingByPointer = false;
            pending = null;
            var length = Math.hypot(vx, vy) || 1;
            moveVX = vx / length;
            moveVY = vy / length;
            player.x = Phaser.Math.Clamp(player.x + moveVX * step, 38, worldWidth - 38);
            player.y = Phaser.Math.Clamp(player.y + moveVY * step, 50, worldHeight - 38);
            state.set('Moving');
          } else if (movingByPointer) {
            var dx = destination.x - player.x;
            var dy = destination.y - player.y;
            var distance = Math.hypot(dx, dy);
            if (distance <= Math.max(10, step)) {
              player.x = destination.x;
              player.y = destination.y;
              movingByPointer = false;
              state.set('Idle');
            } else {
              moveVX = dx / distance;
              moveVY = dy / distance;
              player.x += moveVX * step;
              player.y += moveVY * step;
            }
          }

          root.dataset.worldPlayer = player.x.toFixed(1) + ',' + player.y.toFixed(1);
          root.dataset.worldTarget = destination.x.toFixed(1) + ',' + destination.y.toFixed(1);
          player.setDepth(Math.floor(player.y) + 20);
          if (!reducedMotion && (keyboardMoving || movingByPointer)) playerArt.y = Math.sin(scene.time.now / 80) * 2;
          else playerArt.y = 0;

          var nearest = null;
          var nearestDistance = Infinity;
          objectViews.forEach(function (view) {
            var d = Phaser.Math.Distance.Between(player.x, player.y, view.x, view.y + 45);
            var radius = Math.max(105, Number(view.object.interaction_radius || 8) * 13);
            var isNear = d <= radius;
            view.label.setAlpha(isNear ? 1 : 0);
            if (isNear && d < nearestDistance) { nearest = view; nearestDistance = d; }
          });

          if (nearest !== near) {
            near = nearest;
            if (near) {
              state.set('NearObject', localName(payload, near.object));
              state.showPrompt(near.object, function () { sheet.openObject(near.object); });
            } else {
              state.hidePrompt();
            }
          }

          if (pending && near === pending) {
            movingByPointer = false;
            pending = null;
          }

          pets.forEach(function (pet, index) {
            var lag = 48 + index * 34;
            var targetX = player.x - Math.sign(moveVX || 1) * lag;
            var targetY = player.y + 22 + index * 8;
            pet.x = Phaser.Math.Linear(pet.x, targetX, reducedMotion ? 0.3 : 0.075);
            pet.y = Phaser.Math.Linear(pet.y, targetY, reducedMotion ? 0.3 : 0.075);
            pet.setDepth(Math.floor(pet.y));
          });
        });

        root.querySelectorAll('[data-world-fallback-object]').forEach(function (button) {
          button.addEventListener('click', function () {
            var found = objectViews.find(function (view) { return view.object.id === button.dataset.worldFallbackObject; });
            if (found) moveTo(found.x, found.y + 88, found);
          });
        });

        root.classList.add('world-engine-ready');
        root.classList.remove('world-engine-loading');
        state.set('Spawn');
        window.setTimeout(function () { if (state.current() === 'Spawn') state.set('Idle'); }, reducedMotion ? 0 : 900);
      }
    };

    state.set('Loading');
    root.classList.add('world-engine-loading');
    runtime.game = new Phaser.Game({
      type: Phaser.AUTO,
      parent: host,
      width: 1280,
      height: 720,
      backgroundColor: '#dce4df',
      transparent: false,
      pixelArt: true,
      roundPixels: true,
      render: { antialias: false, powerPreference: lowPower ? 'low-power' : 'high-performance' },
      scale: { mode: Phaser.Scale.RESIZE, autoCenter: Phaser.Scale.CENTER_BOTH },
      physics: { default: 'arcade', arcade: { gravity: { y: 0 }, debug: false } },
      scene: sceneConfig,
      audio: { noAudio: true }
    });

    function offline() { state.set('Offline'); }
    function online() { if (state.current() === 'Offline') state.set('Idle'); }
    window.addEventListener('offline', offline);
    window.addEventListener('online', online);

    runtime.destroy = function () {
      runtime.destroyed = true;
      window.removeEventListener('offline', offline);
      window.removeEventListener('online', online);
      if (runtime.game) runtime.game.destroy(true);
      instances.delete(host);
    };
    return runtime;
  }

  function boot(host) {
    if (instances.has(host) || host.dataset.worldBooting === 'true') return;
    host.dataset.worldBooting = 'true';
    var root = host.closest('[data-world-runtime]');
    if (!root) return;
    var payload;
    try { payload = parsePayload(host); }
    catch (error) {
      root.dataset.worldState = 'Error';
      host.dataset.worldBooting = 'false';
      return;
    }

    Promise.all([ensurePhaser(), payload.bundle.scene.kind === 'home' ? ensureHomeLayout() : Promise.resolve(null)])
      .then(function (result) {
        if (!host.isConnected) return;
        instances.set(host, createWorld(host, root, payload, result[1], result[0]));
      })
      .catch(function (error) {
        console.error('[inspace world]', error);
        root.classList.remove('world-engine-loading');
        root.dataset.worldState = 'Error';
        var status = root.querySelector('[data-world-status]');
        if (status) status.textContent = text(payload, '场景加载失败，可使用下方文字入口', 'Scene failed to load; use the text links below');
      })
      .finally(function () { host.dataset.worldBooting = 'false'; });
  }

  function scan() {
    document.querySelectorAll('[data-world-canvas]').forEach(boot);
    instances.forEach(function (runtime, host) {
      if (!host.isConnected) runtime.destroy();
    });
  }

  var observer = new MutationObserver(scan);
  observer.observe(document.documentElement, { childList: true, subtree: true });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', scan, { once: true });
  else scan();
  window.addEventListener('instant-space-hydrated', scan);
})();
