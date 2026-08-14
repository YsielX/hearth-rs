return {
    api_version = 1, id = "ICC_069", name = "Ghastly Conjurer",
    text = "<b>Battlecry:</b> Add a 'Mirror Image' spell to your hand.",
    set = "ICECROWN", type = "minion", class = "mage", rarity = "rare",
    cost = 4, attack = 2, health = 6, tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self) ctx:give_card(ctx:controller(self), "CS2_027") end,
}
