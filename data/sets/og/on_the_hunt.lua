local card = {
    api_version = 1, id = "OG_061", name = "On the Hunt",
    text = "Deal $1 damage.\nSummon a 1/1 Mastiff.", set = "OG", type = "spell",
    class = "hunter", rarity = "common", cost = 1, target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}
function card.on_play(ctx, self, target)
    ctx:damage(target, 1)
    ctx:summon(ctx:controller(self), "OG_061t")
end
card.tokens = {{ id = "OG_061t", name = "Mastiff", text = "", set = "OG",
    type = "minion", class = "hunter", cost = 1, attack = 1, health = 1,
    tags = { "beast" } }}
return card
