local card = {
    api_version = 1, id = "OG_122", name = "Mukla, Tyrant of the Vale",
    text = "<b>Battlecry:</b> Add 2 Bananas to your hand.", set = "OG", type = "minion",
    rarity = "legendary", cost = 6, attack = 5, health = 5, tags = { "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        cardlib.effects.give_card(ctx, player, "EX1_014t"); cardlib.effects.give_card(ctx, player, "EX1_014t")
    end,
}
card.tokens = {{
    id = "EX1_014t", name = "Bananas", text = "Give a minion +1/+1.",
    set = "EXPERT1", type = "spell", cost = 1, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end,
}}
return card
