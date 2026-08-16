local card = {
    api_version = 1, id = "AT_022", name = "Fist of Jaraxxus",
    text = "When you play or discard this, deal $4 damage to a random enemy.", set = "TGT", type = "spell",
    class = "warlock", rarity = "rare", cost = 4, spell_school = "fel",
}

local function request_damage(ctx, self)
    ctx:random_entity(ctx:enemy_characters(self), "deal_random_damage")
end

function card.on_play(ctx, self) request_damage(ctx, self) end
function card.deal_random_damage(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end

card.triggers = {{
    event = "card_discarded", timing = "after", active_zones = { "graveyard" },
    condition = function(ctx, self, event) return event.entity == self end,
    effect = request_damage,
}}

return card
