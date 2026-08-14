local card = {
    api_version = 1,
    id = "BRM_012",
    name = "Fireguard Destroyer",
    text = "<b>Battlecry:</b> Gain 1-4 Attack. <b>Overload:</b> (1)",
    set = "BRM",
    type = "minion",
    class = "shaman",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 6,
    tags = { "elemental" },
    keywords = { "battlecry", "overload" },
    keyword_params = { overload = 1 },
}

function card.on_battlecry(ctx, self)
    ctx:random_value({ 1, 2, 3, 4 }, "gain_attack")
end

function card.gain_attack(ctx, self, amount)
    ctx:buff(self, amount, 0)
end

return card
