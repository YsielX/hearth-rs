local card = {
    api_version = 1, id = "AT_050", name = "Charged Hammer",
    text = "<b>Deathrattle:</b> Your Hero Power becomes 'Deal 2 damage.'", set = "TGT", type = "weapon",
    class = "shaman", rarity = "epic", cost = 3, attack = 2, health = 3,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    ctx:replace_hero_power(ctx:controller(self), "AT_050t")
end

card.tokens = {{
    id = "AT_050t", name = "Lightning Jolt", text = "Deal $2 damage.", set = "TGT",
    type = "hero_power", class = "shaman", cost = 2, target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end,
}}

return card
