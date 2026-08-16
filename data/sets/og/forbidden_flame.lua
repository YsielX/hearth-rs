local card = {
    api_version = 1, id = "OG_086", name = "Forbidden Flame",
    text = "Spend all your Mana. Deal that much damage to a minion.",
    set = "OG", type = "spell", class = "mage", rarity = "epic",
    spell_school = "fire", cost = 0, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    local player = ctx:controller(self)
    local amount = ctx:player(player).mana
    ctx:spend_mana(player, amount)
    cardlib.effects.damage(ctx, target, amount)
end
return card
