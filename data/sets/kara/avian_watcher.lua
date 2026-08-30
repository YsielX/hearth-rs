return {
    api_version = 1,
    id = "KAR_037",
    name = "Avian Watcher",
    text = "<b>Battlecry:</b> If you control a <b>Secret</b>, gain +1/+1\nand <b>Taunt</b>.",
    set = "KARA",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 3,
    health = 6,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        if #ctx:secrets(ctx:controller(self)) > 0 then
            cardlib.effects.buff(ctx, self, 1, 1)
            cardlib.effects.grant_keyword(ctx, self, "taunt")
        end
    end,
}
