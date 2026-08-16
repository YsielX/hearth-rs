return {
    api_version = 1,
    id = "AT_011",
    name = "Holy Champion",
    text = "<b>Overheal:</b> Gain +2 Attack.",
    set = "TGT",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 2,
    attack = 1,
    health = 4,
    keywords = { "overheal" },
    on_overheal = function(ctx, self, amount)
        cardlib.effects.modify(ctx, self, { stat = "attack", operation = "add", value = 2 })
    end,
}
