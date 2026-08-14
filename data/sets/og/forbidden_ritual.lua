local card = {
    api_version = 1, id = "OG_114", name = "Forbidden Ritual",
    text = "Spend all your Mana. Summon that many 1/1 Tentacles.", set = "OG", type = "spell",
    class = "warlock", rarity = "rare", cost = 0, spell_school = "shadow",
}
function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local mana = ctx:player(player).mana
    if mana > 0 then ctx:spend_mana(player, mana) end
    local count = math.min(mana, 7 - #ctx:board(player))
    for _ = 1, count do ctx:summon(player, "OG_114a") end
end
card.tokens = {{ id = "OG_114a", name = "Icky Tentacle", text = "", set = "OG",
    type = "minion", class = "warlock", cost = 1, attack = 1, health = 1 }}
return card
