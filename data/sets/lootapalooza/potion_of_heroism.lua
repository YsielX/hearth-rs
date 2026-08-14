local card = {
    api_version = 1, id = "LOOT_088", name = "Potion of Heroism",
    text = "Give a minion <b>Divine Shield</b>.\nDraw a card.", set = "LOOTAPALOOZA",
    type = "spell", class = "paladin", rarity = "common", spell_school = "holy",
    cost = 2, target_mode = "required", targets = function(ctx) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    ctx:grant_keyword(target, "divine_shield")
    ctx:draw(ctx:controller(self), 1)
end
return card
