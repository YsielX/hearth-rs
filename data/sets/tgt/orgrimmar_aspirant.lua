return {
    api_version = 1, id = "AT_066", name = "Orgrimmar Aspirant",
    text = "<b>Inspire:</b> Give your weapon +1 Attack.", set = "TGT", type = "minion",
    class = "warrior", rarity = "common", cost = 3, attack = 3, health = 3,
    keywords = { "inspire" }, on_inspire = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon ~= nil then ctx:modify(weapon, { stat = "attack", operation = "add", value = 1 }) end
    end,
}
