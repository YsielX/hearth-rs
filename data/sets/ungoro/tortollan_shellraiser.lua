local card = {
    api_version = 1, id = "UNG_037", name = "Tortollan Shellraiser",
    text = "[x]<b>Taunt</b>\n<b>Deathrattle:</b> Give a random\n friendly minion +1/+1.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "common",
    cost = 3, attack = 2, health = 5, keywords = { "taunt", "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "buff_shellraiser_friend") end
end
function card.buff_shellraiser_friend(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end
return card
