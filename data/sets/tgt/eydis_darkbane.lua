local card = {
    api_version = 1, id = "AT_131", name = "Eydis Darkbane",
    text = "Whenever <b>you</b> target this minion with a spell, deal 3 damage to a random enemy.",
    set = "TGT", type = "minion", rarity = "legendary", cost = 3, attack = 3, health = 4,
    tags = { "undead" },
}

card.triggers = {{
    event = "spell_targeted", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.target == self
    end,
    effect = function(ctx, self) ctx:random_entity(ctx:enemy_characters(self), "deal_darkbane_damage") end,
}}

function card.deal_darkbane_damage(ctx, self, target) cardlib.effects.damage(ctx, target, 3) end

return card
