## Установка

1. Установить vJoy драйвер. На момент написания readme, последняя версия с поддержкой windows 10 и windows 11 - https://github.com/BrunnerInnovation/vJoy/releases.
2. Запустить приложение.
3. Настроить джойстик в игре.
4. Перейти на сайт с клиентом.
5. Настроить раскладку.
6. Проверить через vJoy monitor или в игре.

## Dev

Сервер обрабатывает действия


### API

#### Button

```json
{
    "button": {
        "button": 1, // id кнопки,
        "pressed": true
    }
}
```

#### Axis

```json
{
    "axis": {
        "axis": "x", // x, y, z, rx, rx, rz, slider, dialslider
        "value": 0.5 // диапазон 0 - 1
    }
}
```
